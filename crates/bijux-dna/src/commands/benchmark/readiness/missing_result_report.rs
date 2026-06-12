use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::expected_benchmark_results::{
    collect_expected_benchmark_result_rows, ExpectedBenchmarkResultRow,
};
use crate::commands::benchmark::local_stage_result_manifest::{
    load_validated_stage_result_manifest_path, validate_stage_result_manifest,
    BenchStageResultCommandV1, BenchStageResultManifestV1, BenchStageResultOutputV1,
    BenchStageResultResourceMetricSource, BenchStageResultResourceMetricsV1,
    BenchStageResultRuntimeV1, BenchStageResultStatus, BenchStageResultToolV1,
    BENCH_STAGE_RESULT_SCHEMA_VERSION,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_MISSING_RESULT_REPORT_TEST_PATH: &str =
    "benchmarks/readiness/missing-result-report-test.json";
const DEFAULT_MISSING_RESULT_REPORT_FIXTURE_ROOT: &str =
    "benchmarks/readiness/missing-result-report-fixture";
const EXPECTED_RESULT_ROOT_PREFIX: &str = "runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/";
const MISSING_RESULT_REPORT_SCHEMA_VERSION: &str = "bijux.bench.readiness.missing_result_report.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MissingResultStatus {
    Present,
    MissingResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MissingResultReportRow {
    pub(crate) result_row_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) fixture_id: String,
    pub(crate) sample_scope: String,
    pub(crate) expected_manifest_path: String,
    pub(crate) audit_manifest_path: String,
    pub(crate) result_status: MissingResultStatus,
    pub(crate) expected_output_artifact_ids: Vec<String>,
    pub(crate) observed_output_artifact_ids: Vec<String>,
    pub(crate) normalized_metrics_output_id: Option<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MissingResultReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) fake_result_root: String,
    pub(crate) expected_row_count: usize,
    pub(crate) present_result_row_count: usize,
    pub(crate) missing_result_row_count: usize,
    pub(crate) passes_behavior_test: bool,
    pub(crate) removed_result_row_id: String,
    pub(crate) removed_manifest_path: String,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<MissingResultReportRow>,
}

#[derive(Debug, Clone)]
struct MissingResultFixture {
    fake_result_root: PathBuf,
    removed_result_row_id: String,
    removed_manifest_path: PathBuf,
}

pub(crate) fn run_render_missing_result_report(
    args: &parse::BenchReadinessRenderMissingResultReportArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_missing_result_report(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_MISSING_RESULT_REPORT_TEST_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_missing_result_report(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<MissingResultReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let fake_result_root =
        repo_root.join(PathBuf::from(DEFAULT_MISSING_RESULT_REPORT_FIXTURE_ROOT));
    let expected_rows = collect_expected_benchmark_result_rows(repo_root)?;
    let fixture = materialize_missing_result_fixture(repo_root, &fake_result_root, &expected_rows)?;
    let rows =
        collect_missing_result_report_rows(repo_root, &fixture.fake_result_root, &expected_rows)?;

    let expected_row_count = rows.len();
    let present_result_row_count =
        rows.iter().filter(|row| row.result_status == MissingResultStatus::Present).count();
    let missing_result_row_count =
        rows.iter().filter(|row| row.result_status == MissingResultStatus::MissingResult).count();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
    }

    let report = MissingResultReport {
        schema_version: MISSING_RESULT_REPORT_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        fake_result_root: path_relative_to_repo(repo_root, &fixture.fake_result_root),
        expected_row_count,
        present_result_row_count,
        missing_result_row_count,
        passes_behavior_test: false,
        removed_result_row_id: fixture.removed_result_row_id.clone(),
        removed_manifest_path: path_relative_to_repo(repo_root, &fixture.removed_manifest_path),
        domain_counts,
        rows,
    };
    let report = ensure_missing_result_report_contract(report)?;
    let payload =
        serde_json::to_string_pretty(&report).context("render missing-result report to JSON")?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(&output_path, payload.as_bytes())?;
    Ok(report)
}

fn materialize_missing_result_fixture(
    repo_root: &Path,
    fake_result_root: &Path,
    expected_rows: &[ExpectedBenchmarkResultRow],
) -> Result<MissingResultFixture> {
    if fake_result_root.exists() {
        fs::remove_dir_all(fake_result_root)
            .with_context(|| format!("remove {}", fake_result_root.display()))?;
    }
    fs::create_dir_all(fake_result_root)
        .with_context(|| format!("create {}", fake_result_root.display()))?;

    let removed_row = expected_rows
        .iter()
        .find(|row| {
            row.domain == "fastq"
                && row.stage_id == "fastq.screen_taxonomy"
                && row.tool_id == "kraken2"
                && row.fixture_id == "corpus-02-edna-mini"
                && row.sample_scope == "sample-set"
        })
        .ok_or_else(|| {
            anyhow!("missing governed result row for the missing-result behavior test")
        })?;

    for row in expected_rows {
        write_fake_result_fixture_row(repo_root, fake_result_root, row)?;
    }
    let removed_manifest_path = fixture_manifest_path(fake_result_root, removed_row)?;
    fs::remove_file(&removed_manifest_path)
        .with_context(|| format!("remove {}", removed_manifest_path.display()))?;

    Ok(MissingResultFixture {
        fake_result_root: fake_result_root.to_path_buf(),
        removed_result_row_id: removed_row.result_row_id.clone(),
        removed_manifest_path,
    })
}

fn write_fake_result_fixture_row(
    repo_root: &Path,
    fake_result_root: &Path,
    row: &ExpectedBenchmarkResultRow,
) -> Result<()> {
    if row.expected_output_artifact_ids.is_empty() {
        return Err(anyhow!(
            "missing-result report requires at least one expected output artifact for `{}`",
            row.result_row_id
        ));
    }
    let result_root = fixture_result_root(fake_result_root, row)?;
    let manifest_path = result_root.join("stage-result.json");
    fs::create_dir_all(&result_root)
        .with_context(|| format!("create {}", result_root.display()))?;

    let stdout_path = result_root.join("stdout.log");
    let stderr_path = result_root.join("stderr.log");
    fs::write(
        &stdout_path,
        format!("fake benchmark stdout\nstage_id={}\ntool_id={}\n", row.stage_id, row.tool_id),
    )
    .with_context(|| format!("write {}", stdout_path.display()))?;
    fs::write(
        &stderr_path,
        format!("fake benchmark stderr\nstage_id={}\ntool_id={}\n", row.stage_id, row.tool_id),
    )
    .with_context(|| format!("write {}", stderr_path.display()))?;

    let outputs = row
        .expected_output_artifact_ids
        .iter()
        .map(|artifact_id| build_fake_output_entry(repo_root, &result_root, row, artifact_id))
        .collect::<Result<Vec<_>>>()?;
    let manifest = BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: row.stage_id.clone(),
        tool: BenchStageResultToolV1 { id: row.tool_id.clone() },
        command: BenchStageResultCommandV1 {
            rendered: format!(
                "bijux-dna bench readiness missing-result-fixture --stage {} --tool {}",
                row.stage_id, row.tool_id
            ),
        },
        runtime: BenchStageResultRuntimeV1 {
            mode: "fake_benchmark_result".to_string(),
            status: BenchStageResultStatus::Succeeded,
            started_at: "1970-01-01T00:00:00Z".to_string(),
            finished_at: "1970-01-01T00:00:01Z".to_string(),
            elapsed_seconds: 1.0,
            exit_code: 0,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::Estimated,
            memory_mb: Some(512.0),
            cpu_threads: Some(1),
        },
        outputs,
    };
    validate_stage_result_manifest(&manifest)
        .with_context(|| format!("validate {}", manifest_path.display()))?;
    let payload =
        serde_json::to_vec_pretty(&manifest).context("encode fake benchmark result manifest")?;
    bijux_dna_infra::atomic_write_bytes(&manifest_path, &payload)?;
    Ok(())
}

fn build_fake_output_entry(
    repo_root: &Path,
    result_root: &Path,
    row: &ExpectedBenchmarkResultRow,
    artifact_id: &str,
) -> Result<BenchStageResultOutputV1> {
    let declared_path = format!("outputs/{artifact_id}.json");
    let absolute_output_path = result_root.join(&declared_path);
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(
        &absolute_output_path,
        format!(
            "{{\"artifact_id\":\"{artifact_id}\",\"stage_id\":\"{}\",\"tool_id\":\"{}\"}}\n",
            row.stage_id, row.tool_id
        ),
    )
    .with_context(|| format!("write {}", absolute_output_path.display()))?;
    Ok(BenchStageResultOutputV1 {
        artifact_id: artifact_id.to_string(),
        declared_path: declared_path.clone(),
        realized_path: path_relative_to_repo(repo_root, &absolute_output_path),
        role: if row.normalized_metrics_output_id.as_deref() == Some(artifact_id) {
            "normalized_metrics".to_string()
        } else {
            "benchmark_output".to_string()
        },
        optional: false,
        exists: true,
    })
}

fn collect_missing_result_report_rows(
    repo_root: &Path,
    fake_result_root: &Path,
    expected_rows: &[ExpectedBenchmarkResultRow],
) -> Result<Vec<MissingResultReportRow>> {
    let mut rows = Vec::with_capacity(expected_rows.len());
    for row in expected_rows {
        let audit_manifest_path = fixture_manifest_path(fake_result_root, row)?;
        let audit_manifest_path_label = path_relative_to_repo(repo_root, &audit_manifest_path);
        if !audit_manifest_path.is_file() {
            rows.push(MissingResultReportRow {
                result_row_id: row.result_row_id.clone(),
                domain: row.domain.clone(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                fixture_id: row.fixture_id.clone(),
                sample_scope: row.sample_scope.clone(),
                expected_manifest_path: row.stage_result_manifest_path.clone(),
                audit_manifest_path: audit_manifest_path_label,
                result_status: MissingResultStatus::MissingResult,
                expected_output_artifact_ids: row.expected_output_artifact_ids.clone(),
                observed_output_artifact_ids: Vec::new(),
                normalized_metrics_output_id: row.normalized_metrics_output_id.clone(),
                reason: format!(
                    "expected benchmark result row `{}` is retained in the report even though `{}` is missing",
                    row.result_row_id, row.stage_result_manifest_path
                ),
            });
            continue;
        }

        let manifest = load_validated_stage_result_manifest_path(&audit_manifest_path)
            .with_context(|| format!("load {}", audit_manifest_path.display()))?;
        if manifest.stage_id != row.stage_id || manifest.tool.id != row.tool_id {
            return Err(anyhow!(
                "fake benchmark result manifest `{}` drifted from `{}` / `{}`",
                audit_manifest_path.display(),
                row.stage_id,
                row.tool_id
            ));
        }
        let observed_output_artifact_ids = manifest
            .outputs
            .iter()
            .map(|artifact| artifact.artifact_id.clone())
            .collect::<Vec<_>>();
        let observed_output_set =
            observed_output_artifact_ids.iter().cloned().collect::<BTreeSet<_>>();
        let expected_output_set =
            row.expected_output_artifact_ids.iter().cloned().collect::<BTreeSet<_>>();
        if observed_output_set != expected_output_set {
            return Err(anyhow!(
                "fake benchmark result manifest `{}` drifted from expected artifact ids for `{}`",
                audit_manifest_path.display(),
                row.result_row_id
            ));
        }
        rows.push(MissingResultReportRow {
            result_row_id: row.result_row_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            fixture_id: row.fixture_id.clone(),
            sample_scope: row.sample_scope.clone(),
            expected_manifest_path: row.stage_result_manifest_path.clone(),
            audit_manifest_path: audit_manifest_path_label,
            result_status: MissingResultStatus::Present,
            expected_output_artifact_ids: row.expected_output_artifact_ids.clone(),
            observed_output_artifact_ids,
            normalized_metrics_output_id: row.normalized_metrics_output_id.clone(),
            reason: format!(
                "expected benchmark result row `{}` remains present with a validated stage-result manifest",
                row.result_row_id
            ),
        });
    }
    Ok(rows)
}

fn ensure_missing_result_report_contract(
    mut report: MissingResultReport,
) -> Result<MissingResultReport> {
    if report.rows.len() != 116 {
        return Err(anyhow!(
            "missing-result report must retain exactly 116 expected benchmark rows, found {}",
            report.rows.len()
        ));
    }
    if report.expected_row_count != 116 {
        return Err(anyhow!(
            "missing-result report must track exactly 116 expected rows, found {}",
            report.expected_row_count
        ));
    }
    if report.present_result_row_count != 115 {
        return Err(anyhow!(
            "missing-result report must retain exactly 115 present benchmark rows after removing one result, found {}",
            report.present_result_row_count
        ));
    }
    if report.missing_result_row_count != 1 {
        return Err(anyhow!(
            "missing-result report must emit exactly one missing_result row, found {}",
            report.missing_result_row_count
        ));
    }
    let removed_row = report
        .rows
        .iter()
        .find(|row| row.result_row_id == report.removed_result_row_id)
        .ok_or_else(|| anyhow!("missing removed result row `{}`", report.removed_result_row_id))?;
    if removed_row.result_status != MissingResultStatus::MissingResult
        || removed_row.domain != "fastq"
        || removed_row.stage_id != "fastq.screen_taxonomy"
        || removed_row.tool_id != "kraken2"
        || removed_row.fixture_id != "corpus-02-edna-mini"
        || removed_row.sample_scope != "sample-set"
    {
        return Err(anyhow!(
            "missing-result report must keep the governed taxonomy row visible as missing_result"
        ));
    }
    if report
        .rows
        .iter()
        .filter(|row| row.result_status == MissingResultStatus::MissingResult)
        .count()
        != 1
    {
        return Err(anyhow!("missing-result report must not hide or duplicate missing rows"));
    }
    report.passes_behavior_test = true;
    Ok(report)
}

fn fixture_result_root(
    fake_result_root: &Path,
    row: &ExpectedBenchmarkResultRow,
) -> Result<PathBuf> {
    let suffix = row.result_root.strip_prefix(EXPECTED_RESULT_ROOT_PREFIX).ok_or_else(|| {
        anyhow!(
            "expected benchmark result root `{}` does not start with `{EXPECTED_RESULT_ROOT_PREFIX}`",
            row.result_root
        )
    })?;
    Ok(fake_result_root.join(suffix))
}

fn fixture_manifest_path(
    fake_result_root: &Path,
    row: &ExpectedBenchmarkResultRow,
) -> Result<PathBuf> {
    Ok(fixture_result_root(fake_result_root, row)?.join("stage-result.json"))
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
        render_missing_result_report, MissingResultStatus, DEFAULT_MISSING_RESULT_REPORT_TEST_PATH,
        MISSING_RESULT_REPORT_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn missing_result_report_tracks_removed_expected_result() {
        let root = repo_root();
        let report = render_missing_result_report(
            &root,
            PathBuf::from(DEFAULT_MISSING_RESULT_REPORT_TEST_PATH),
        )
        .expect("render missing-result report");

        assert_eq!(report.schema_version, MISSING_RESULT_REPORT_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_MISSING_RESULT_REPORT_TEST_PATH);
        assert_eq!(report.fake_result_root, "benchmarks/readiness/missing-result-report-fixture");
        assert_eq!(report.expected_row_count, 116);
        assert_eq!(report.present_result_row_count, 115);
        assert_eq!(report.missing_result_row_count, 1);
        assert!(report.passes_behavior_test);
        assert_eq!(report.domain_counts.get("fastq").copied(), Some(67));
        assert_eq!(report.domain_counts.get("bam").copied(), Some(49));

        let removed_row = report
            .rows
            .iter()
            .find(|row| row.result_row_id == report.removed_result_row_id)
            .expect("removed result row");
        assert_eq!(removed_row.result_status, MissingResultStatus::MissingResult);
        assert_eq!(removed_row.stage_id, "fastq.screen_taxonomy");
        assert_eq!(removed_row.tool_id, "kraken2");
        assert!(removed_row.observed_output_artifact_ids.is_empty());
        assert!(report
            .rows
            .iter()
            .all(|row| row.result_status == MissingResultStatus::MissingResult
                || !row.observed_output_artifact_ids.is_empty()));
    }
}
