use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use bijux_dna_bench_model::validate_benchmark_stage_result_manifest;
pub(crate) use bijux_dna_bench_model::{
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

use crate::commands::cli::parse;
use crate::commands::cli::render;

const BENCH_STAGE_RESULT_VALIDATION_SCHEMA_VERSION: &str = "bijux.bench.stage_result_validation.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct BenchStageResultValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) manifest_path: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) output_count: usize,
    pub(crate) status: BenchStageResultStatus,
    pub(crate) valid: bool,
}

pub(crate) fn run_validate_stage_result(
    args: &parse::BenchLocalValidateStageResultArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let manifest_path = if args.manifest.is_absolute() {
        args.manifest.clone()
    } else {
        repo_root.join(&args.manifest)
    };
    let report = validate_stage_result_manifest_path(&repo_root, &manifest_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.manifest_path);
    }
    Ok(())
}

pub(crate) fn validate_stage_result_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<BenchStageResultValidationReport> {
    let manifest = load_validated_stage_result_manifest_path(manifest_path)?;
    Ok(BenchStageResultValidationReport {
        schema_version: BENCH_STAGE_RESULT_VALIDATION_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        stage_id: manifest.stage_id,
        tool_id: manifest.tool.id,
        output_count: manifest.outputs.len(),
        status: manifest.runtime.status,
        valid: true,
    })
}

pub(crate) fn load_validated_stage_result_manifest_path(
    manifest_path: &Path,
) -> Result<BenchStageResultManifestV1> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let manifest: BenchStageResultManifestV1 =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))?;
    validate_stage_result_manifest(&manifest)
        .with_context(|| format!("validate {}", manifest_path.display()))?;
    Ok(manifest)
}

pub(crate) fn validate_stage_result_manifest(manifest: &BenchStageResultManifestV1) -> Result<()> {
    validate_benchmark_stage_result_manifest(manifest).map_err(Into::into)
}

pub(crate) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        validate_stage_result_manifest, BenchStageResultCommandV1, BenchStageResultManifestV1,
        BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
        BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
        BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
    };

    fn valid_manifest() -> BenchStageResultManifestV1 {
        BenchStageResultManifestV1 {
            schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
            stage_id: "fastq.report_qc".to_string(),
            tool: BenchStageResultToolV1 { id: "multiqc".to_string() },
            command: BenchStageResultCommandV1 { rendered: "echo ok".to_string() },
            runtime: BenchStageResultRuntimeV1 {
                mode: "fake_run".to_string(),
                status: BenchStageResultStatus::Succeeded,
                started_at: "1970-01-01T00:00:00Z".to_string(),
                finished_at: "1970-01-01T00:00:01Z".to_string(),
                elapsed_seconds: 1.0,
                exit_code: 0,
            },
            resource_metrics: BenchStageResultResourceMetricsV1 {
                source: BenchStageResultResourceMetricSource::Estimated,
                memory_mb: Some(128.0),
                cpu_threads: Some(1),
            },
            outputs: vec![BenchStageResultOutputV1 {
                artifact_id: "report_json".to_string(),
                declared_path: "declared".to_string(),
                realized_path: "realized".to_string(),
                role: "report".to_string(),
                optional: false,
                exists: true,
            }],
        }
    }

    #[test]
    fn stage_result_manifest_accepts_valid_required_fields() {
        let manifest = valid_manifest();
        validate_stage_result_manifest(&manifest).expect("valid stage-result manifest");
    }

    #[test]
    fn stage_result_manifest_rejects_missing_required_fields() {
        let cases = vec![
            (
                "command",
                json!({
                    "schema_version": BENCH_STAGE_RESULT_SCHEMA_VERSION,
                    "stage_id": "fastq.report_qc",
                    "tool": {"id": "multiqc"},
                    "runtime": {
                        "mode": "fake_run",
                        "status": "succeeded",
                        "started_at": "1970-01-01T00:00:00Z",
                        "finished_at": "1970-01-01T00:00:01Z",
                        "elapsed_seconds": 1.0,
                        "exit_code": 0
                    },
                    "resource_metrics": {
                        "source": "estimated",
                        "memory_mb": 128.0,
                        "cpu_threads": 1
                    },
                    "outputs": [{
                        "artifact_id": "report_json",
                        "declared_path": "declared",
                        "realized_path": "realized",
                        "role": "report",
                        "optional": false,
                        "exists": true
                    }]
                }),
            ),
            (
                "tool",
                json!({
                    "schema_version": BENCH_STAGE_RESULT_SCHEMA_VERSION,
                    "stage_id": "fastq.report_qc",
                    "command": {"rendered": "echo ok"},
                    "runtime": {
                        "mode": "fake_run",
                        "status": "succeeded",
                        "started_at": "1970-01-01T00:00:00Z",
                        "finished_at": "1970-01-01T00:00:01Z",
                        "elapsed_seconds": 1.0,
                        "exit_code": 0
                    },
                    "resource_metrics": {
                        "source": "estimated",
                        "memory_mb": 128.0,
                        "cpu_threads": 1
                    },
                    "outputs": [{
                        "artifact_id": "report_json",
                        "declared_path": "declared",
                        "realized_path": "realized",
                        "role": "report",
                        "optional": false,
                        "exists": true
                    }]
                }),
            ),
            (
                "runtime",
                json!({
                    "schema_version": BENCH_STAGE_RESULT_SCHEMA_VERSION,
                    "stage_id": "fastq.report_qc",
                    "tool": {"id": "multiqc"},
                    "command": {"rendered": "echo ok"},
                    "resource_metrics": {
                        "source": "estimated",
                        "memory_mb": 128.0,
                        "cpu_threads": 1
                    },
                    "outputs": [{
                        "artifact_id": "report_json",
                        "declared_path": "declared",
                        "realized_path": "realized",
                        "role": "report",
                        "optional": false,
                        "exists": true
                    }]
                }),
            ),
            (
                "outputs",
                json!({
                    "schema_version": BENCH_STAGE_RESULT_SCHEMA_VERSION,
                    "stage_id": "fastq.report_qc",
                    "tool": {"id": "multiqc"},
                    "command": {"rendered": "echo ok"},
                    "runtime": {
                        "mode": "fake_run",
                        "status": "succeeded",
                        "started_at": "1970-01-01T00:00:00Z",
                        "finished_at": "1970-01-01T00:00:01Z",
                        "elapsed_seconds": 1.0,
                        "exit_code": 0
                    },
                    "resource_metrics": {
                        "source": "estimated",
                        "memory_mb": 128.0,
                        "cpu_threads": 1
                    }
                }),
            ),
            (
                "resource_metrics",
                json!({
                    "schema_version": BENCH_STAGE_RESULT_SCHEMA_VERSION,
                    "stage_id": "fastq.report_qc",
                    "tool": {"id": "multiqc"},
                    "command": {"rendered": "echo ok"},
                    "runtime": {
                        "mode": "fake_run",
                        "status": "succeeded",
                        "started_at": "1970-01-01T00:00:00Z",
                        "finished_at": "1970-01-01T00:00:01Z",
                        "elapsed_seconds": 1.0,
                        "exit_code": 0
                    },
                    "outputs": [{
                        "artifact_id": "report_json",
                        "declared_path": "declared",
                        "realized_path": "realized",
                        "role": "report",
                        "optional": false,
                        "exists": true
                    }]
                }),
            ),
        ];

        for (missing_field, payload) in cases {
            let error = serde_json::from_value::<BenchStageResultManifestV1>(payload)
                .expect_err("missing required field should fail parse");
            assert!(
                error.to_string().contains(missing_field),
                "parse failure should identify missing `{missing_field}`: {error}"
            );
        }
    }

    #[test]
    fn stage_result_manifest_rejects_invalid_resource_metric_shapes() {
        let mut missing_values = valid_manifest();
        missing_values.resource_metrics.source = BenchStageResultResourceMetricSource::Estimated;
        missing_values.resource_metrics.memory_mb = None;
        missing_values.resource_metrics.cpu_threads = None;
        let error = validate_stage_result_manifest(&missing_values)
            .expect_err("estimated resource metrics must declare at least one value");
        assert!(error.to_string().contains("at least one resource metric"));

        let mut not_available_with_values = valid_manifest();
        not_available_with_values.resource_metrics.source =
            BenchStageResultResourceMetricSource::NotAvailable;
        let error = validate_stage_result_manifest(&not_available_with_values)
            .expect_err("not_available resource metrics must not carry values");
        assert!(error.to_string().contains("must leave `resource_metrics.memory_mb`"));
    }
}
