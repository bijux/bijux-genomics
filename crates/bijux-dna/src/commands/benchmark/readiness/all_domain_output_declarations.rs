use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_expected_benchmark_results::collect_all_domain_expected_benchmark_result_rows;
use super::bam_adapter_output_contract::{
    collect_bam_adapter_output_contract_rows, BamAdapterOutputContractStatus,
};
use super::expected_benchmark_results::collect_expected_benchmark_result_rows;
use super::fastq_adapter_output_contract::{
    collect_fastq_adapter_output_contract_rows, FastqAdapterOutputContractStatus,
};
use super::vcf_adapter_output_coverage::{
    collect_vcf_adapter_output_coverage_rows, VcfAdapterOutputCoverageStatus,
};
use super::vcf_expected_benchmark_results::{
    collect_vcf_expected_benchmark_result_rows, VcfExpectedBenchmarkResultRow,
};
use crate::commands::benchmark::local_slurm_run_paths::LOCAL_SLURM_DRY_RUN_RUN_ID;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_OUTPUT_DECLARATIONS_PATH: &str =
    "benchmarks/readiness/output-declarations-all-domains.tsv";
const ALL_DOMAIN_OUTPUT_DECLARATIONS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_output_declarations.v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    domain: String,
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AllDomainOutputDeclarationStatus {
    Complete,
    Incomplete,
    MissingAdapter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainOutputDeclarationRow {
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) raw_outputs: Vec<String>,
    pub(crate) normalized_metrics: Vec<String>,
    pub(crate) logs: Vec<String>,
    pub(crate) manifest: String,
    pub(crate) index_outputs: Vec<String>,
    pub(crate) status: AllDomainOutputDeclarationStatus,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainOutputDeclarationsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) result_id_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) incomplete_row_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) status_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<AllDomainOutputDeclarationRow>,
}

pub(crate) fn run_render_all_domain_output_declarations(
    args: &parse::BenchReadinessRenderAllDomainOutputDeclarationsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_output_declarations(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_OUTPUT_DECLARATIONS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_output_declarations(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainOutputDeclarationsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_all_domain_output_declaration_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_all_domain_output_declarations_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let row_count = rows.len();
    let result_id_count =
        rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>().len();
    let complete_row_count =
        rows.iter().filter(|row| row.status == AllDomainOutputDeclarationStatus::Complete).count();
    let incomplete_row_count = row_count.saturating_sub(complete_row_count);
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *status_counts.entry(status_label(row.status).to_string()).or_default() += 1;
    }

    Ok(AllDomainOutputDeclarationsReport {
        schema_version: ALL_DOMAIN_OUTPUT_DECLARATIONS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count,
        result_id_count,
        complete_row_count,
        incomplete_row_count,
        domain_counts,
        status_counts,
        rows,
    })
}

pub(crate) fn collect_all_domain_output_declaration_rows(
    repo_root: &Path,
) -> Result<Vec<AllDomainOutputDeclarationRow>> {
    let canonical_rows = collect_all_domain_expected_benchmark_result_rows(repo_root)?;
    let fastq_bam_expected_by_result_id = collect_expected_benchmark_result_rows(repo_root)?
        .into_iter()
        .map(|row| (row.result_row_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let fastq_contract_by_binding = collect_fastq_adapter_output_contract_rows(repo_root)?
        .into_iter()
        .map(|row| (binding_key("fastq", &row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();
    let bam_contract_by_binding = collect_bam_adapter_output_contract_rows(repo_root)?
        .into_iter()
        .map(|row| (binding_key("bam", &row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();
    let vcf_output_by_binding = collect_vcf_adapter_output_coverage_rows(repo_root)?
        .into_iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .map(|row| (binding_key("vcf", &row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();
    let vcf_expected_by_result_id = collect_vcf_expected_benchmark_result_rows(repo_root)?
        .into_iter()
        .map(|row| (vcf_result_id(&row), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::with_capacity(canonical_rows.len());
    for canonical in canonical_rows {
        match canonical.domain.as_str() {
            "fastq" => {
                let expected = fastq_bam_expected_by_result_id.get(&canonical.result_id).ok_or_else(
                    || {
                        anyhow!(
                            "all-domain output declarations are missing FASTQ expected-result coverage for `{}`",
                            canonical.result_id
                        )
                    },
                )?;
                let contract = fastq_contract_by_binding
                    .get(&binding_key("fastq", &canonical.stage_id, &canonical.tool_id))
                    .ok_or_else(|| {
                        anyhow!(
                            "all-domain output declarations are missing FASTQ adapter output coverage for `{}` / `{}`",
                            canonical.stage_id,
                            canonical.tool_id
                        )
                    })?;
                rows.push(AllDomainOutputDeclarationRow {
                    result_id: canonical.result_id,
                    domain: canonical.domain,
                    stage_id: canonical.stage_id,
                    tool_id: canonical.tool_id,
                    corpus_id: canonical.corpus_id,
                    asset_profile_id: canonical.asset_profile_id,
                    raw_outputs: expected.raw_output_artifact_ids.clone(),
                    normalized_metrics: expected
                        .normalized_metrics_output_id
                        .clone()
                        .into_iter()
                        .collect(),
                    logs: vec![
                        format!("stdout={}", expected.stdout_path),
                        format!("stderr={}", expected.stderr_path),
                    ],
                    manifest: expected.stage_result_manifest_path.clone(),
                    index_outputs: Vec::new(),
                    status: fastq_status(contract.output_contract_status),
                });
            }
            "bam" => {
                let expected = fastq_bam_expected_by_result_id.get(&canonical.result_id).ok_or_else(
                    || {
                        anyhow!(
                            "all-domain output declarations are missing BAM expected-result coverage for `{}`",
                            canonical.result_id
                        )
                    },
                )?;
                let contract = bam_contract_by_binding
                    .get(&binding_key("bam", &canonical.stage_id, &canonical.tool_id))
                    .ok_or_else(|| {
                        anyhow!(
                            "all-domain output declarations are missing BAM adapter output coverage for `{}` / `{}`",
                            canonical.stage_id,
                            canonical.tool_id
                        )
                    })?;
                rows.push(AllDomainOutputDeclarationRow {
                    result_id: canonical.result_id,
                    domain: canonical.domain,
                    stage_id: canonical.stage_id,
                    tool_id: canonical.tool_id,
                    corpus_id: canonical.corpus_id,
                    asset_profile_id: canonical.asset_profile_id,
                    raw_outputs: expected.raw_output_artifact_ids.clone(),
                    normalized_metrics: expected
                        .normalized_metrics_output_id
                        .clone()
                        .into_iter()
                        .collect(),
                    logs: vec![
                        format!("stdout={}", expected.stdout_path),
                        format!("stderr={}", expected.stderr_path),
                    ],
                    manifest: expected.stage_result_manifest_path.clone(),
                    index_outputs: Vec::new(),
                    status: bam_status(contract.output_contract_status),
                });
            }
            "vcf" => {
                let expected = vcf_expected_by_result_id.get(&canonical.result_id).ok_or_else(|| {
                    anyhow!(
                        "all-domain output declarations are missing VCF expected-result coverage for `{}`",
                        canonical.result_id
                    )
                })?;
                let coverage = vcf_output_by_binding
                    .get(&binding_key("vcf", &canonical.stage_id, &canonical.tool_id))
                    .ok_or_else(|| {
                        anyhow!(
                            "all-domain output declarations are missing VCF output coverage for `{}` / `{}`",
                            canonical.stage_id,
                            canonical.tool_id
                        )
                    })?;
                let result_root = vcf_result_root(expected);
                rows.push(AllDomainOutputDeclarationRow {
                    result_id: canonical.result_id,
                    domain: canonical.domain,
                    stage_id: canonical.stage_id,
                    tool_id: canonical.tool_id,
                    corpus_id: canonical.corpus_id,
                    asset_profile_id: canonical.asset_profile_id,
                    raw_outputs: coverage
                        .raw_outputs
                        .iter()
                        .map(|entry| artifact_id(entry))
                        .collect(),
                    normalized_metrics: coverage
                        .normalized_metrics
                        .iter()
                        .map(|entry| artifact_id(entry))
                        .collect(),
                    logs: vec![
                        format!("stdout={result_root}/stdout.log"),
                        format!("stderr={result_root}/stderr.log"),
                    ],
                    manifest: format!("{result_root}/stage-result.json"),
                    index_outputs: coverage
                        .index_outputs
                        .iter()
                        .map(|entry| artifact_id(entry))
                        .collect(),
                    status: vcf_status(coverage.status),
                });
            }
            other => {
                return Err(anyhow!(
                    "all-domain output declarations do not support legacy domain `{other}`"
                ));
            }
        }
    }

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });
    ensure_all_domain_output_declaration_contract(&rows)?;
    Ok(rows)
}

fn ensure_all_domain_output_declaration_contract(
    rows: &[AllDomainOutputDeclarationRow],
) -> Result<()> {
    if rows.len() != 120 {
        return Err(anyhow!(
            "all-domain output declarations must retain exactly 120 benchmark-ready rows, found {}",
            rows.len()
        ));
    }

    let mut seen_result_ids = BTreeSet::<&str>::new();
    for row in rows {
        if row.result_id.trim().is_empty()
            || row.domain.trim().is_empty()
            || row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.corpus_id.trim().is_empty()
            || row.asset_profile_id.trim().is_empty()
            || row.manifest.trim().is_empty()
        {
            return Err(anyhow!(
                "all-domain output declaration row `{}` is missing required columns",
                row.result_id
            ));
        }
        if !seen_result_ids.insert(row.result_id.as_str()) {
            return Err(anyhow!(
                "all-domain output declarations contain duplicate result_id `{}`",
                row.result_id
            ));
        }
        if row.raw_outputs.is_empty()
            || row.normalized_metrics.is_empty()
            || row.logs.len() != 2
            || row.logs.iter().any(|entry| entry.trim().is_empty())
        {
            return Err(anyhow!(
                "all-domain output declaration row `{}` is missing governed output, normalized-metrics, or log declarations",
                row.result_id
            ));
        }
        if row.status != AllDomainOutputDeclarationStatus::Complete {
            return Err(anyhow!(
                "all-domain output declaration row `{}` is not complete",
                row.result_id
            ));
        }
    }

    require_output_row(
        rows,
        "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2",
        "classification_report_json",
        "screen_report_tsv",
        "target/slurm-dry-run/runs/local-benchmark-dry-run/corpus-02-edna-mini/fastq.screen_taxonomy/sample-set/kraken2/stage-result.json",
        None,
    )?;
    require_output_row(
        rows,
        "bam:corpus-01-kinship-mini:bam.kinship:sample-set:king",
        "kinship_report",
        "summary",
        "target/slurm-dry-run/runs/local-benchmark-dry-run/corpus-01-kinship-mini/bam.kinship/sample-set/king/stage-result.json",
        None,
    )?;
    require_output_row(
        rows,
        "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools",
        "called_vcf",
        "called_vcf",
        "target/slurm-dry-run/runs/local-benchmark-dry-run/vcf_production_regression/vcf.call/bam_bundle/bcftools/stage-result.json",
        Some("called_vcf_tbi"),
    )?;

    Ok(())
}

fn require_output_row(
    rows: &[AllDomainOutputDeclarationRow],
    result_id: &str,
    normalized_metric: &str,
    raw_output: &str,
    manifest: &str,
    index_output: Option<&str>,
) -> Result<()> {
    let row = rows
        .iter()
        .find(|row| row.result_id == result_id)
        .ok_or_else(|| anyhow!("all-domain output declarations are missing `{result_id}`"))?;
    if !row.normalized_metrics.iter().any(|candidate| candidate == normalized_metric)
        || !row.raw_outputs.iter().any(|candidate| candidate == raw_output)
        || row.manifest != manifest
        || !row.logs.iter().any(|entry| {
            entry
                == &format!(
                    "stdout={}",
                    manifest.trim_end_matches("/stage-result.json").to_string() + "/stdout.log"
                )
        })
        || !row.logs.iter().any(|entry| {
            entry
                == &format!(
                    "stderr={}",
                    manifest.trim_end_matches("/stage-result.json").to_string() + "/stderr.log"
                )
        })
        || index_output.is_some_and(|required| {
            !row.index_outputs.iter().any(|candidate| candidate == required)
        })
    {
        return Err(anyhow!(
            "all-domain output declaration row `{result_id}` drifted from the governed output contract"
        ));
    }
    Ok(())
}

fn render_all_domain_output_declarations_tsv(rows: &[AllDomainOutputDeclarationRow]) -> String {
    let mut rendered = String::from(
        "result_id\tdomain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\traw_outputs\tnormalized_metrics\tlogs\tmanifest\tindex_outputs\tstatus\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.result_id),
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.corpus_id),
            sanitize_tsv(&row.asset_profile_id),
            sanitize_tsv(&row.raw_outputs.join(",")),
            sanitize_tsv(&row.normalized_metrics.join(",")),
            sanitize_tsv(&row.logs.join(",")),
            sanitize_tsv(&row.manifest),
            sanitize_tsv(&row.index_outputs.join(",")),
            sanitize_tsv(status_label(row.status)),
        ));
    }
    rendered
}

fn fastq_status(status: FastqAdapterOutputContractStatus) -> AllDomainOutputDeclarationStatus {
    match status {
        FastqAdapterOutputContractStatus::Complete => AllDomainOutputDeclarationStatus::Complete,
        FastqAdapterOutputContractStatus::Incomplete => {
            AllDomainOutputDeclarationStatus::Incomplete
        }
        FastqAdapterOutputContractStatus::MissingAdapter => {
            AllDomainOutputDeclarationStatus::MissingAdapter
        }
    }
}

fn bam_status(status: BamAdapterOutputContractStatus) -> AllDomainOutputDeclarationStatus {
    match status {
        BamAdapterOutputContractStatus::Complete => AllDomainOutputDeclarationStatus::Complete,
        BamAdapterOutputContractStatus::Incomplete => AllDomainOutputDeclarationStatus::Incomplete,
        BamAdapterOutputContractStatus::MissingAdapter => {
            AllDomainOutputDeclarationStatus::MissingAdapter
        }
    }
}

fn vcf_status(status: VcfAdapterOutputCoverageStatus) -> AllDomainOutputDeclarationStatus {
    match status {
        VcfAdapterOutputCoverageStatus::Complete => AllDomainOutputDeclarationStatus::Complete,
        VcfAdapterOutputCoverageStatus::Incomplete => AllDomainOutputDeclarationStatus::Incomplete,
    }
}

fn artifact_id(entry: &str) -> String {
    entry
        .split_once('=')
        .map(|(artifact_id, _)| artifact_id.to_string())
        .unwrap_or_else(|| entry.to_string())
}

fn vcf_result_root(row: &VcfExpectedBenchmarkResultRow) -> String {
    format!(
        "target/slurm-dry-run/runs/{}/{}/{}/{}/{}",
        LOCAL_SLURM_DRY_RUN_RUN_ID, row.corpus_id, row.stage_id, row.asset_profile_id, row.tool_id
    )
}

fn vcf_result_id(row: &VcfExpectedBenchmarkResultRow) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        row.domain, row.corpus_id, row.stage_id, row.asset_profile_id, row.tool_id
    )
}

fn binding_key(domain: &str, stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
    }
}

fn status_label(status: AllDomainOutputDeclarationStatus) -> &'static str {
    match status {
        AllDomainOutputDeclarationStatus::Complete => "complete",
        AllDomainOutputDeclarationStatus::Incomplete => "incomplete",
        AllDomainOutputDeclarationStatus::MissingAdapter => "missing_adapter",
    }
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

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}
