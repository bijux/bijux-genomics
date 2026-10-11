use super::paths::{fixture_manifest_path, fixture_result_root, result_row_id};
use anyhow::Context;
use anyhow::{anyhow, Result};
use bijux_dna_bench_model::{
    validate_benchmark_stage_result_manifest as validate_stage_result_manifest,
    BenchmarkStageResultCommand as BenchStageResultCommandV1,
    BenchmarkStageResultManifest as BenchStageResultManifestV1,
    BenchmarkStageResultOutput as BenchStageResultOutputV1,
    BenchmarkStageResultResourceMetricSource as BenchStageResultResourceMetricSource,
    BenchmarkStageResultResourceMetrics as BenchStageResultResourceMetricsV1,
    BenchmarkStageResultRuntime as BenchStageResultRuntimeV1,
    BenchmarkStageResultStatus as BenchStageResultStatus,
    BenchmarkStageResultTool as BenchStageResultToolV1,
    BENCHMARK_STAGE_RESULT_SCHEMA_V2 as BENCH_STAGE_RESULT_SCHEMA_VERSION,
};
use bijux_dna_bench_vcf_evidence::model::expectations::VcfExpectedBenchmarkResultRow;
use std::path::{Path, PathBuf};
#[derive(Debug, Clone)]
pub(super) struct VcfMissingResultFixture {
    pub(super) fake_result_root: PathBuf,
    pub(super) removed_result_row_id: String,
    pub(super) removed_manifest_path: PathBuf,
}

pub(super) fn materialize_vcf_missing_result_fixture(
    fake_result_root: &Path,
    expected_rows: &[VcfExpectedBenchmarkResultRow],
) -> Result<VcfMissingResultFixture> {
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
    bijux_dna_infra::remove_file(&removed_manifest_path)
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
    bijux_dna_infra::ensure_dir(&result_root)
        .with_context(|| format!("create {}", result_root.display()))?;

    let stdout_path = result_root.join("stdout.log");
    let stderr_path = result_root.join("stderr.log");
    bijux_dna_infra::write_bytes(
        &stdout_path,
        format!("fake vcf benchmark stdout\nstage_id={}\ntool_id={}\n", row.stage_id, row.tool_id),
    )
    .with_context(|| format!("write {}", stdout_path.display()))?;
    bijux_dna_infra::write_bytes(
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
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::write_bytes(
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
