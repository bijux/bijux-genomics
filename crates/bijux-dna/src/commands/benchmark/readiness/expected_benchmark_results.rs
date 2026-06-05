use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::bam_adapter_output_contract::{
    collect_bam_adapter_output_contract_rows, BamAdapterOutputContractRow,
    BamAdapterOutputContractStatus,
};
use super::bam_corpus_assignment::collect_bam_corpus_assignment_rows;
use super::fastq_adapter_output_contract::{
    collect_fastq_adapter_output_contract_rows, FastqAdapterOutputContractRow,
    FastqAdapterOutputContractStatus,
};
use super::fastq_corpus_assignment::{
    collect_fastq_corpus_assignment_rows, FastqCorpusAssignmentStatus,
};
use crate::commands::benchmark::local_slurm_run_paths::{
    load_fixture_sample_scope_map, LOCAL_SLURM_DRY_RUN_RUN_ID,
};
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH: &str =
    "target/bench-readiness/expected-benchmark-results.tsv";
const EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.expected_benchmark_results.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ExpectedBenchmarkResultRow {
    pub(crate) result_row_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) corpus_family_id: String,
    pub(crate) fixture_id: String,
    pub(crate) sample_scope: String,
    pub(crate) expected_output_artifact_ids: Vec<String>,
    pub(crate) raw_output_artifact_ids: Vec<String>,
    pub(crate) normalized_metrics_output_id: Option<String>,
    pub(crate) result_root: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) stdout_path: String,
    pub(crate) stderr_path: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ExpectedBenchmarkResultsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) row_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) fixture_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<ExpectedBenchmarkResultRow>,
}

pub(crate) fn run_render_expected_benchmark_results(
    args: &parse::BenchReadinessRenderExpectedBenchmarkResultsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_expected_benchmark_results(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_expected_benchmark_results(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<ExpectedBenchmarkResultsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_expected_benchmark_result_rows(repo_root)?;
    let stage_count = rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut fixture_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *fixture_counts.entry(row.fixture_id.clone()).or_default() += 1;
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_expected_benchmark_results_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(ExpectedBenchmarkResultsReport {
        schema_version: EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        tool_count,
        row_count: rows.len(),
        domain_counts,
        fixture_counts,
        rows,
    })
}

pub(crate) fn collect_expected_benchmark_result_rows(
    repo_root: &Path,
) -> Result<Vec<ExpectedBenchmarkResultRow>> {
    let fixture_sample_scopes = load_fixture_sample_scope_map(repo_root)?;
    let fastq_contract_rows = collect_fastq_adapter_output_contract_rows(repo_root)?;
    let bam_contract_rows = collect_bam_adapter_output_contract_rows(repo_root)?;
    let (_, _, fastq_corpus_rows) = collect_fastq_corpus_assignment_rows(repo_root)?;
    let (_, _, bam_corpus_rows) = collect_bam_corpus_assignment_rows(repo_root)?;
    let fastq_readiness_kinds = load_stage_readiness_kinds(repo_root, BenchLocalDomain::Fastq)?;
    let bam_readiness_kinds = load_stage_readiness_kinds(repo_root, BenchLocalDomain::Bam)?;

    let fastq_contract_by_binding = fastq_contract_rows
        .into_iter()
        .map(|row| ((row.stage_id.clone(), row.tool_id.clone()), row))
        .collect::<BTreeMap<_, _>>();
    let bam_contract_by_binding = bam_contract_rows
        .into_iter()
        .map(|row| ((row.stage_id.clone(), row.tool_id.clone()), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::new();

    for row in fastq_corpus_rows.into_iter().filter(|row| {
        row.benchmark_status == "benchmark_ready"
            && row.assignment_status == FastqCorpusAssignmentStatus::Assigned
    }) {
        let fixture_id = row.fixture_id.clone().ok_or_else(|| {
            anyhow!(
                "FASTQ benchmark-ready row `{}` / `{}` is missing fixture_id",
                row.stage_id,
                row.tool_id
            )
        })?;
        let corpus_family_id = row.corpus_family_id.clone().ok_or_else(|| {
            anyhow!(
                "FASTQ benchmark-ready row `{}` / `{}` is missing corpus_family_id",
                row.stage_id,
                row.tool_id
            )
        })?;
        let readiness_kind = fastq_readiness_kinds
            .get(&row.stage_id)
            .cloned()
            .ok_or_else(|| anyhow!("missing FASTQ stage readiness kind for `{}`", row.stage_id))?;
        let contract_row = fastq_contract_by_binding
            .get(&(row.stage_id.clone(), row.tool_id.clone()))
            .ok_or_else(|| {
                anyhow!(
                    "missing FASTQ adapter output contract row for `{}` / `{}`",
                    row.stage_id,
                    row.tool_id
                )
            })?;
        rows.push(build_fastq_expected_result_row(
            &row.stage_id,
            &row.tool_id,
            &readiness_kind,
            &corpus_family_id,
            &fixture_id,
            &fixture_sample_scopes,
            contract_row,
        )?);
    }

    for row in bam_corpus_rows.into_iter().filter(|row| row.benchmark_status == "benchmark_ready") {
        let readiness_kind = bam_readiness_kinds
            .get(&row.stage_id)
            .cloned()
            .ok_or_else(|| anyhow!("missing BAM stage readiness kind for `{}`", row.stage_id))?;
        let contract_row = bam_contract_by_binding
            .get(&(row.stage_id.clone(), row.tool_id.clone()))
            .ok_or_else(|| {
                anyhow!(
                    "missing BAM adapter output contract row for `{}` / `{}`",
                    row.stage_id,
                    row.tool_id
                )
            })?;
        rows.push(build_bam_expected_result_row(
            &row.stage_id,
            &row.tool_id,
            &readiness_kind,
            &row.corpus_family_id,
            &row.fixture_id,
            &fixture_sample_scopes,
            contract_row,
        )?);
    }

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.fixture_id.cmp(&right.fixture_id))
    });
    ensure_expected_result_contract(&rows)?;
    Ok(rows)
}

fn load_stage_readiness_kinds(
    repo_root: &Path,
    domain: BenchLocalDomain,
) -> Result<BTreeMap<String, String>> {
    Ok(load_local_stage_inventory(repo_root, domain)?
        .stages
        .into_iter()
        .map(|stage| (stage.stage_id, stage.readiness_kind.as_str().to_string()))
        .collect())
}

fn build_fastq_expected_result_row(
    stage_id: &str,
    tool_id: &str,
    readiness_kind: &str,
    corpus_family_id: &str,
    fixture_id: &str,
    fixture_sample_scopes: &BTreeMap<String, String>,
    contract_row: &FastqAdapterOutputContractRow,
) -> Result<ExpectedBenchmarkResultRow> {
    if contract_row.output_contract_status != FastqAdapterOutputContractStatus::Complete {
        return Err(anyhow!(
            "FASTQ benchmark-ready row `{stage_id}` / `{tool_id}` is missing a complete adapter output contract"
        ));
    }
    let sample_scope = fixture_sample_scopes
        .get(fixture_id)
        .cloned()
        .ok_or_else(|| anyhow!("missing FASTQ fixture sample scope for `{fixture_id}`"))?;
    let result_root = result_root_path(fixture_id, stage_id, &sample_scope, tool_id);
    let stage_result_manifest_path = format!("{result_root}/stage-result.json");
    let stdout_path = format!("{result_root}/stdout.log");
    let stderr_path = format!("{result_root}/stderr.log");
    let normalized_metrics_output_id = contract_row.normalized_metrics_output_id.clone();
    let expected_output_artifact_ids = contract_row.execution_expected_output_ids.clone();
    let raw_output_artifact_ids = contract_row.raw_output_artifact_ids.clone();

    Ok(ExpectedBenchmarkResultRow {
        result_row_id: result_row_id("fastq", fixture_id, stage_id, &sample_scope, tool_id),
        domain: "fastq".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        readiness_kind: readiness_kind.to_string(),
        corpus_family_id: corpus_family_id.to_string(),
        fixture_id: fixture_id.to_string(),
        sample_scope,
        expected_output_artifact_ids,
        raw_output_artifact_ids,
        normalized_metrics_output_id: normalized_metrics_output_id.clone(),
        result_root: result_root.clone(),
        stage_result_manifest_path: stage_result_manifest_path.clone(),
        stdout_path,
        stderr_path,
        reason: format!(
            "benchmark row `{stage_id}` / `{tool_id}` on fixture `{fixture_id}` expects `{stage_result_manifest_path}` with normalized metrics artifact `{}`",
            normalized_metrics_output_id.unwrap_or_else(|| "not_declared".to_string())
        ),
    })
}

fn build_bam_expected_result_row(
    stage_id: &str,
    tool_id: &str,
    readiness_kind: &str,
    corpus_family_id: &str,
    fixture_id: &str,
    fixture_sample_scopes: &BTreeMap<String, String>,
    contract_row: &BamAdapterOutputContractRow,
) -> Result<ExpectedBenchmarkResultRow> {
    if contract_row.output_contract_status != BamAdapterOutputContractStatus::Complete {
        return Err(anyhow!(
            "BAM benchmark-ready row `{stage_id}` / `{tool_id}` is missing a complete adapter output contract"
        ));
    }
    let sample_scope = fixture_sample_scopes
        .get(fixture_id)
        .cloned()
        .ok_or_else(|| anyhow!("missing BAM fixture sample scope for `{fixture_id}`"))?;
    let result_root = result_root_path(fixture_id, stage_id, &sample_scope, tool_id);
    let stage_result_manifest_path = format!("{result_root}/stage-result.json");
    let stdout_path = format!("{result_root}/stdout.log");
    let stderr_path = format!("{result_root}/stderr.log");
    let normalized_metrics_output_id = contract_row.normalized_metrics_output_id.clone();
    let expected_output_artifact_ids = contract_row.execution_expected_output_ids.clone();
    let raw_output_artifact_ids = contract_row.raw_output_artifact_ids.clone();

    Ok(ExpectedBenchmarkResultRow {
        result_row_id: result_row_id("bam", fixture_id, stage_id, &sample_scope, tool_id),
        domain: "bam".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        readiness_kind: readiness_kind.to_string(),
        corpus_family_id: corpus_family_id.to_string(),
        fixture_id: fixture_id.to_string(),
        sample_scope,
        expected_output_artifact_ids,
        raw_output_artifact_ids,
        normalized_metrics_output_id: normalized_metrics_output_id.clone(),
        result_root: result_root.clone(),
        stage_result_manifest_path: stage_result_manifest_path.clone(),
        stdout_path,
        stderr_path,
        reason: format!(
            "benchmark row `{stage_id}` / `{tool_id}` on fixture `{fixture_id}` expects `{stage_result_manifest_path}` with normalized metrics artifact `{}`",
            normalized_metrics_output_id.unwrap_or_else(|| "not_declared".to_string())
        ),
    })
}

fn ensure_expected_result_contract(rows: &[ExpectedBenchmarkResultRow]) -> Result<()> {
    if rows.len() != 112 {
        return Err(anyhow!(
            "expected benchmark result table must retain exactly 112 benchmark-ready rows, found {}",
            rows.len()
        ));
    }
    let unique_result_row_ids =
        rows.iter().map(|row| row.result_row_id.clone()).collect::<BTreeSet<_>>();
    if unique_result_row_ids.len() != rows.len() {
        return Err(anyhow!(
            "expected benchmark result table must keep unique result_row_id values"
        ));
    }
    ensure_row(
        rows,
        "fastq",
        "fastq.screen_taxonomy",
        "kraken2",
        "corpus-02-edna-mini",
        "sample-set",
        Some("classification_report_json"),
    )?;
    ensure_row(
        rows,
        "fastq",
        "fastq.profile_reads",
        "seqkit_stats",
        "corpus-01-mini",
        "sample-set",
        Some("qc_json"),
    )?;
    ensure_row(
        rows,
        "bam",
        "bam.damage",
        "ngsbriggs",
        "corpus-01-adna-damage-mini",
        "adna_damage_non_udg",
        Some("damage_report"),
    )?;
    ensure_row(
        rows,
        "bam",
        "bam.kinship",
        "king",
        "corpus-01-kinship-mini",
        "sample-set",
        Some("kinship_report"),
    )?;
    Ok(())
}

fn ensure_row(
    rows: &[ExpectedBenchmarkResultRow],
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    fixture_id: &str,
    sample_scope: &str,
    normalized_metrics_output_id: Option<&str>,
) -> Result<()> {
    let row = rows
        .iter()
        .find(|row| row.domain == domain && row.stage_id == stage_id && row.tool_id == tool_id)
        .ok_or_else(|| {
            anyhow!("missing expected benchmark result row for `{stage_id}` / `{tool_id}`")
        })?;
    if row.fixture_id != fixture_id
        || row.sample_scope != sample_scope
        || row.normalized_metrics_output_id.as_deref() != normalized_metrics_output_id
    {
        return Err(anyhow!(
            "expected benchmark result row `{stage_id}` / `{tool_id}` drifted from its governed result contract"
        ));
    }
    Ok(())
}

fn result_row_id(
    domain: &str,
    fixture_id: &str,
    stage_id: &str,
    sample_scope: &str,
    tool_id: &str,
) -> String {
    format!("{domain}:{fixture_id}:{stage_id}:{sample_scope}:{tool_id}")
}

fn result_root_path(fixture_id: &str, stage_id: &str, sample_scope: &str, tool_id: &str) -> String {
    format!(
        "target/slurm-dry-run/runs/{}/{}/{}/{}/{}",
        LOCAL_SLURM_DRY_RUN_RUN_ID, fixture_id, stage_id, sample_scope, tool_id
    )
}

fn render_expected_benchmark_results_tsv(rows: &[ExpectedBenchmarkResultRow]) -> String {
    let mut rendered = String::from(
        "result_row_id\tdomain\tstage_id\ttool_id\treadiness_kind\tcorpus_family_id\tfixture_id\tsample_scope\texpected_output_artifact_ids\traw_output_artifact_ids\tnormalized_metrics_output_id\tresult_root\tstage_result_manifest_path\tstdout_path\tstderr_path\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.result_row_id),
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.readiness_kind),
            sanitize_tsv(&row.corpus_family_id),
            sanitize_tsv(&row.fixture_id),
            sanitize_tsv(&row.sample_scope),
            sanitize_tsv(&row.expected_output_artifact_ids.join(",")),
            sanitize_tsv(&row.raw_output_artifact_ids.join(",")),
            sanitize_tsv(row.normalized_metrics_output_id.as_deref().unwrap_or("")),
            sanitize_tsv(&row.result_root),
            sanitize_tsv(&row.stage_result_manifest_path),
            sanitize_tsv(&row.stdout_path),
            sanitize_tsv(&row.stderr_path),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_expected_benchmark_results, DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH,
        EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn expected_benchmark_results_report_tracks_governed_result_rows() {
        let root = repo_root();
        let report = render_expected_benchmark_results(
            &root,
            PathBuf::from(DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH),
        )
        .expect("render expected benchmark results");

        assert_eq!(report.schema_version, EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH);
        assert_eq!(report.row_count, 112);
        assert_eq!(report.stage_count, 47);
        assert_eq!(report.rows.len(), 112);
        assert_eq!(report.domain_counts.get("fastq").copied(), Some(63));
        assert_eq!(report.domain_counts.get("bam").copied(), Some(49));
        assert!(report.rows.iter().any(|row| {
            row.domain == "fastq"
                && row.stage_id == "fastq.screen_taxonomy"
                && row.tool_id == "kraken2"
                && row.fixture_id == "corpus-02-edna-mini"
                && row.sample_scope == "sample-set"
                && row.normalized_metrics_output_id.as_deref() == Some("classification_report_json")
                && row
                    .stage_result_manifest_path
                    == "target/slurm-dry-run/runs/local-benchmark-dry-run/corpus-02-edna-mini/fastq.screen_taxonomy/sample-set/kraken2/stage-result.json"
        }));
        assert!(report.rows.iter().any(|row| {
            row.domain == "bam"
                && row.stage_id == "bam.damage"
                && row.tool_id == "ngsbriggs"
                && row.fixture_id == "corpus-01-adna-damage-mini"
                && row.sample_scope == "adna_damage_non_udg"
                && row.normalized_metrics_output_id.as_deref() == Some("damage_report")
        }));
        assert!(report.rows.iter().all(|row| !row.result_row_id.is_empty()));
    }
}
