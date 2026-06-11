use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::vcf_expected_benchmark_results::{
    collect_vcf_expected_benchmark_result_rows, VcfExpectedBenchmarkResultRow,
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

pub(crate) const DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH: &str =
    "benchmarks/readiness/vcf-missing-result-report-test.json";
const DEFAULT_VCF_MISSING_RESULT_REPORT_FIXTURE_ROOT: &str =
    "benchmarks/readiness/vcf-missing-result-report-fixture";
const VCF_MISSING_RESULT_REPORT_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_missing_result_report.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum VcfMissingResultStatus {
    Present,
    MissingResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfMissingResultReportRow {
    pub(crate) result_row_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) expected_manifest_path: String,
    pub(crate) audit_manifest_path: String,
    pub(crate) result_status: VcfMissingResultStatus,
    pub(crate) expected_output_artifact_ids: Vec<String>,
    pub(crate) observed_output_artifact_ids: Vec<String>,
    pub(crate) expected_metrics: Vec<String>,
    pub(crate) report_section: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfMissingResultReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) fake_result_root: String,
    pub(crate) expected_row_count: usize,
    pub(crate) present_result_row_count: usize,
    pub(crate) missing_result_row_count: usize,
    pub(crate) passes_behavior_test: bool,
    pub(crate) removed_result_row_id: String,
    pub(crate) removed_manifest_path: String,
    pub(crate) report_section_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<VcfMissingResultReportRow>,
}

#[derive(Debug, Clone)]
struct VcfMissingResultFixture {
    fake_result_root: PathBuf,
    removed_result_row_id: String,
    removed_manifest_path: PathBuf,
}

pub(crate) fn run_render_vcf_missing_result_report(
    args: &parse::BenchReadinessRenderVcfMissingResultReportArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_missing_result_report(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_missing_result_report(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfMissingResultReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let fake_result_root =
        repo_root.join(PathBuf::from(DEFAULT_VCF_MISSING_RESULT_REPORT_FIXTURE_ROOT));
    let expected_rows = collect_vcf_expected_benchmark_result_rows(repo_root)?;
    let fixture = materialize_vcf_missing_result_fixture(&fake_result_root, &expected_rows)?;
    let rows = collect_vcf_missing_result_report_rows(
        repo_root,
        &fixture.fake_result_root,
        &expected_rows,
    )?;

    let expected_row_count = rows.len();
    let present_result_row_count =
        rows.iter().filter(|row| row.result_status == VcfMissingResultStatus::Present).count();
    let missing_result_row_count = rows
        .iter()
        .filter(|row| row.result_status == VcfMissingResultStatus::MissingResult)
        .count();
    let mut report_section_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *report_section_counts.entry(row.report_section.clone()).or_default() += 1;
    }

    let report = VcfMissingResultReport {
        schema_version: VCF_MISSING_RESULT_REPORT_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        fake_result_root: path_relative_to_repo(repo_root, &fixture.fake_result_root),
        expected_row_count,
        present_result_row_count,
        missing_result_row_count,
        passes_behavior_test: false,
        removed_result_row_id: fixture.removed_result_row_id.clone(),
        removed_manifest_path: path_relative_to_repo(repo_root, &fixture.removed_manifest_path),
        report_section_counts,
        rows,
    };
    let report = ensure_vcf_missing_result_report_contract(report)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    Ok(report)
}

fn materialize_vcf_missing_result_fixture(
    fake_result_root: &Path,
    expected_rows: &[VcfExpectedBenchmarkResultRow],
) -> Result<VcfMissingResultFixture> {
    if fake_result_root.exists() {
        fs::remove_dir_all(fake_result_root)
            .with_context(|| format!("remove {}", fake_result_root.display()))?;
    }
    fs::create_dir_all(fake_result_root)
        .with_context(|| format!("create {}", fake_result_root.display()))?;

    let removed_row = expected_rows
        .iter()
        .find(|row| {
            row.domain == "vcf"
                && row.stage_id == "vcf.stats"
                && row.tool_id == "bcftools"
                && row.corpus_id == "vcf_production_regression"
                && row.asset_profile_id == "vcf_cohort"
        })
        .ok_or_else(|| {
            anyhow!("missing governed VCF result row for the missing-result behavior test")
        })?;

    for row in expected_rows {
        write_fake_vcf_result_fixture_row(fake_result_root, row)?;
    }
    let removed_manifest_path = fixture_manifest_path(fake_result_root, removed_row);
    fs::remove_file(&removed_manifest_path)
        .with_context(|| format!("remove {}", removed_manifest_path.display()))?;

    Ok(VcfMissingResultFixture {
        fake_result_root: fake_result_root.to_path_buf(),
        removed_result_row_id: result_row_id(removed_row),
        removed_manifest_path,
    })
}

fn write_fake_vcf_result_fixture_row(
    fake_result_root: &Path,
    row: &VcfExpectedBenchmarkResultRow,
) -> Result<()> {
    if row.expected_outputs.is_empty() {
        return Err(anyhow!(
            "VCF missing-result report requires at least one expected output artifact for `{}`",
            result_row_id(row)
        ));
    }
    let result_root = fixture_result_root(fake_result_root, row);
    let manifest_path = result_root.join("stage-result.json");
    fs::create_dir_all(&result_root)
        .with_context(|| format!("create {}", result_root.display()))?;

    let stdout_path = result_root.join("stdout.log");
    let stderr_path = result_root.join("stderr.log");
    fs::write(
        &stdout_path,
        format!("fake vcf benchmark stdout\nstage_id={}\ntool_id={}\n", row.stage_id, row.tool_id),
    )
    .with_context(|| format!("write {}", stdout_path.display()))?;
    fs::write(
        &stderr_path,
        format!("fake vcf benchmark stderr\nstage_id={}\ntool_id={}\n", row.stage_id, row.tool_id),
    )
    .with_context(|| format!("write {}", stderr_path.display()))?;

    let outputs = row
        .expected_outputs
        .iter()
        .map(|artifact_id| build_fake_output_entry(&result_root, artifact_id))
        .collect::<Result<Vec<_>>>()?;
    let manifest = BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: row.stage_id.clone(),
        tool: BenchStageResultToolV1 { id: row.tool_id.clone() },
        command: BenchStageResultCommandV1 {
            rendered: format!(
                "bijux-dna bench readiness render-vcf-missing-result-report --stage {} --tool {}",
                row.stage_id, row.tool_id
            ),
        },
        runtime: BenchStageResultRuntimeV1 {
            mode: "fake_vcf_benchmark_result".to_string(),
            status: BenchStageResultStatus::Succeeded,
            started_at: "1970-01-01T00:00:00Z".to_string(),
            finished_at: "1970-01-01T00:00:01Z".to_string(),
            elapsed_seconds: 1.0,
            exit_code: 0,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::Estimated,
            memory_mb: Some(256.0),
            cpu_threads: Some(1),
        },
        outputs,
    };
    validate_stage_result_manifest(&manifest)
        .with_context(|| format!("validate {}", manifest_path.display()))?;
    bijux_dna_infra::atomic_write_json(&manifest_path, &manifest)?;
    Ok(())
}

fn build_fake_output_entry(
    result_root: &Path,
    artifact_id: &str,
) -> Result<BenchStageResultOutputV1> {
    let declared_path = format!("outputs/{artifact_id}.json");
    let absolute_output_path = result_root.join(&declared_path);
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(
        &absolute_output_path,
        format!("{{\"artifact_id\":\"{artifact_id}\",\"status\":\"present\"}}\n"),
    )
    .with_context(|| format!("write {}", absolute_output_path.display()))?;
    Ok(BenchStageResultOutputV1 {
        artifact_id: artifact_id.to_string(),
        declared_path: declared_path.clone(),
        realized_path: declared_path,
        role: "benchmark_output".to_string(),
        optional: false,
        exists: true,
    })
}

fn collect_vcf_missing_result_report_rows(
    repo_root: &Path,
    fake_result_root: &Path,
    expected_rows: &[VcfExpectedBenchmarkResultRow],
) -> Result<Vec<VcfMissingResultReportRow>> {
    let mut rows = Vec::with_capacity(expected_rows.len());
    for row in expected_rows {
        let manifest_path = fixture_manifest_path(fake_result_root, row);
        let manifest_label = path_relative_to_repo(repo_root, &manifest_path);
        if !manifest_path.is_file() {
            rows.push(VcfMissingResultReportRow {
                result_row_id: result_row_id(row),
                domain: row.domain.clone(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                corpus_id: row.corpus_id.clone(),
                asset_profile_id: row.asset_profile_id.clone(),
                expected_manifest_path: manifest_label.clone(),
                audit_manifest_path: manifest_label,
                result_status: VcfMissingResultStatus::MissingResult,
                expected_output_artifact_ids: row.expected_outputs.clone(),
                observed_output_artifact_ids: Vec::new(),
                expected_metrics: row.expected_metrics.clone(),
                report_section: row.report_section.clone(),
                reason: format!(
                    "expected VCF benchmark row `{}` remains visible even though its manifest is missing",
                    result_row_id(row)
                ),
            });
            continue;
        }

        let manifest = load_validated_stage_result_manifest_path(&manifest_path)
            .with_context(|| format!("load {}", manifest_path.display()))?;
        if manifest.stage_id != row.stage_id || manifest.tool.id != row.tool_id {
            return Err(anyhow!(
                "fake VCF benchmark result manifest `{}` drifted from `{}` / `{}`",
                manifest_path.display(),
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
            observed_output_artifact_ids.iter().cloned().collect::<std::collections::BTreeSet<_>>();
        let expected_output_set =
            row.expected_outputs.iter().cloned().collect::<std::collections::BTreeSet<_>>();
        if observed_output_set != expected_output_set {
            return Err(anyhow!(
                "fake VCF benchmark result manifest `{}` drifted from expected output ids for `{}`",
                manifest_path.display(),
                result_row_id(row)
            ));
        }

        rows.push(VcfMissingResultReportRow {
            result_row_id: result_row_id(row),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            corpus_id: row.corpus_id.clone(),
            asset_profile_id: row.asset_profile_id.clone(),
            expected_manifest_path: manifest_label.clone(),
            audit_manifest_path: manifest_label,
            result_status: VcfMissingResultStatus::Present,
            expected_output_artifact_ids: row.expected_outputs.clone(),
            observed_output_artifact_ids,
            expected_metrics: row.expected_metrics.clone(),
            report_section: row.report_section.clone(),
            reason: format!(
                "expected VCF benchmark row `{}` remains present with a validated stage-result manifest",
                result_row_id(row)
            ),
        });
    }
    Ok(rows)
}

fn ensure_vcf_missing_result_report_contract(
    mut report: VcfMissingResultReport,
) -> Result<VcfMissingResultReport> {
    if report.rows.len() != 15 {
        return Err(anyhow!(
            "VCF missing-result report must retain exactly 15 expected benchmark rows, found {}",
            report.rows.len()
        ));
    }
    if report.expected_row_count != 15 {
        return Err(anyhow!(
            "VCF missing-result report must track exactly 15 expected rows, found {}",
            report.expected_row_count
        ));
    }
    if report.present_result_row_count != 14 {
        return Err(anyhow!(
            "VCF missing-result report must retain exactly 14 present benchmark rows after removing one result, found {}",
            report.present_result_row_count
        ));
    }
    if report.missing_result_row_count != 1 {
        return Err(anyhow!(
            "VCF missing-result report must emit exactly one missing_result row, found {}",
            report.missing_result_row_count
        ));
    }
    let removed_row = report
        .rows
        .iter()
        .find(|row| row.result_row_id == report.removed_result_row_id)
        .ok_or_else(|| {
            anyhow!("missing removed VCF result row `{}`", report.removed_result_row_id)
        })?;
    if removed_row.result_status != VcfMissingResultStatus::MissingResult
        || removed_row.domain != "vcf"
        || removed_row.stage_id != "vcf.stats"
        || removed_row.tool_id != "bcftools"
        || removed_row.corpus_id != "vcf_production_regression"
        || removed_row.asset_profile_id != "vcf_cohort"
    {
        return Err(anyhow!(
            "VCF missing-result report must keep the governed stats row visible as missing_result"
        ));
    }
    if report
        .rows
        .iter()
        .filter(|row| row.result_status == VcfMissingResultStatus::MissingResult)
        .count()
        != 1
    {
        return Err(anyhow!("VCF missing-result report must not hide or duplicate missing rows"));
    }
    report.passes_behavior_test = true;
    Ok(report)
}

fn fixture_result_root(fake_result_root: &Path, row: &VcfExpectedBenchmarkResultRow) -> PathBuf {
    fake_result_root
        .join(&row.corpus_id)
        .join(&row.stage_id)
        .join(&row.asset_profile_id)
        .join(&row.tool_id)
}

fn fixture_manifest_path(fake_result_root: &Path, row: &VcfExpectedBenchmarkResultRow) -> PathBuf {
    fixture_result_root(fake_result_root, row).join("stage-result.json")
}

fn result_row_id(row: &VcfExpectedBenchmarkResultRow) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        row.domain, row.corpus_id, row.stage_id, row.asset_profile_id, row.tool_id
    )
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
        render_vcf_missing_result_report, VcfMissingResultStatus,
        DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH, VCF_MISSING_RESULT_REPORT_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_missing_result_report_tracks_removed_expected_result() {
        let root = repo_root();
        let report = render_vcf_missing_result_report(
            &root,
            PathBuf::from(DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH),
        )
        .expect("render VCF missing-result report");

        assert_eq!(report.schema_version, VCF_MISSING_RESULT_REPORT_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH);
        assert_eq!(
            report.fake_result_root,
            "benchmarks/readiness/vcf-missing-result-report-fixture"
        );
        assert_eq!(report.expected_row_count, 15);
        assert_eq!(report.present_result_row_count, 14);
        assert_eq!(report.missing_result_row_count, 1);
        assert!(report.passes_behavior_test);
        assert_eq!(report.report_section_counts.get("variant_calling").copied(), Some(5));
        assert_eq!(report.report_section_counts.get("quality_control").copied(), Some(2));

        let removed_row = report
            .rows
            .iter()
            .find(|row| row.result_row_id == report.removed_result_row_id)
            .expect("removed result row");
        assert_eq!(removed_row.result_status, VcfMissingResultStatus::MissingResult);
        assert_eq!(removed_row.stage_id, "vcf.stats");
        assert_eq!(removed_row.tool_id, "bcftools");
        assert!(removed_row.observed_output_artifact_ids.is_empty());
        assert!(report.rows.iter().all(|row| {
            row.result_status == VcfMissingResultStatus::MissingResult
                || !row.observed_output_artifact_ids.is_empty()
        }));
    }
}
