use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const BENCH_STAGE_RESULT_SCHEMA_VERSION: &str = "bijux.bench.stage_result.v2";
const BENCH_STAGE_RESULT_VALIDATION_SCHEMA_VERSION: &str = "bijux.bench.stage_result_validation.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BenchStageResultStatus {
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BenchStageResultToolV1 {
    pub(crate) id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BenchStageResultCommandV1 {
    pub(crate) rendered: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BenchStageResultRuntimeV1 {
    pub(crate) mode: String,
    pub(crate) status: BenchStageResultStatus,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BenchStageResultResourceMetricSource {
    Measured,
    Estimated,
    NotAvailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BenchStageResultResourceMetricsV1 {
    pub(crate) source: BenchStageResultResourceMetricSource,
    pub(crate) memory_mb: Option<f64>,
    pub(crate) cpu_threads: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BenchStageResultOutputV1 {
    pub(crate) artifact_id: String,
    pub(crate) declared_path: String,
    pub(crate) realized_path: String,
    pub(crate) role: String,
    pub(crate) optional: bool,
    pub(crate) exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BenchStageResultManifestV1 {
    pub(crate) schema_version: String,
    pub(crate) stage_id: String,
    pub(crate) tool: BenchStageResultToolV1,
    pub(crate) command: BenchStageResultCommandV1,
    pub(crate) runtime: BenchStageResultRuntimeV1,
    pub(crate) resource_metrics: BenchStageResultResourceMetricsV1,
    pub(crate) outputs: Vec<BenchStageResultOutputV1>,
}

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
    let manifest: BenchStageResultManifestV1 = serde_json::from_str(&raw)
        .map_err(|err| anyhow!("parse {}: {err}", manifest_path.display()))?;
    validate_stage_result_manifest(&manifest)
        .with_context(|| format!("validate {}", manifest_path.display()))?;
    Ok(manifest)
}

pub(crate) fn validate_stage_result_manifest(manifest: &BenchStageResultManifestV1) -> Result<()> {
    if manifest.schema_version != BENCH_STAGE_RESULT_SCHEMA_VERSION {
        return Err(anyhow!("unsupported stage-result schema `{}`", manifest.schema_version));
    }
    if manifest.stage_id.trim().is_empty() {
        return Err(anyhow!("stage-result manifest must declare a non-empty `stage_id`"));
    }
    if manifest.tool.id.trim().is_empty() {
        return Err(anyhow!("stage-result manifest must declare a non-empty `tool.id`"));
    }
    if manifest.command.rendered.trim().is_empty() {
        return Err(anyhow!("stage-result manifest must declare a non-empty `command.rendered`"));
    }
    if manifest.runtime.mode.trim().is_empty() {
        return Err(anyhow!("stage-result manifest must declare a non-empty `runtime.mode`"));
    }
    if manifest.runtime.started_at.trim().is_empty() {
        return Err(anyhow!("stage-result manifest must declare a non-empty `runtime.started_at`"));
    }
    if manifest.runtime.finished_at.trim().is_empty() {
        return Err(anyhow!(
            "stage-result manifest must declare a non-empty `runtime.finished_at`"
        ));
    }
    if manifest.runtime.elapsed_seconds.is_sign_negative() {
        return Err(anyhow!(
            "stage-result manifest must declare a non-negative `runtime.elapsed_seconds`"
        ));
    }
    if manifest.resource_metrics.memory_mb.is_some_and(f64::is_sign_negative) {
        return Err(anyhow!(
            "stage-result manifest must declare a non-negative `resource_metrics.memory_mb`"
        ));
    }
    match manifest.resource_metrics.source {
        BenchStageResultResourceMetricSource::Measured
        | BenchStageResultResourceMetricSource::Estimated => {
            if manifest.resource_metrics.memory_mb.is_none()
                && manifest.resource_metrics.cpu_threads.is_none()
            {
                return Err(anyhow!(
                    "stage-result manifest must declare at least one resource metric when `resource_metrics.source` is `{}`",
                    match manifest.resource_metrics.source {
                        BenchStageResultResourceMetricSource::Measured => "measured",
                        BenchStageResultResourceMetricSource::Estimated => "estimated",
                        BenchStageResultResourceMetricSource::NotAvailable => unreachable!(),
                    }
                ));
            }
        }
        BenchStageResultResourceMetricSource::NotAvailable => {
            if manifest.resource_metrics.memory_mb.is_some()
                || manifest.resource_metrics.cpu_threads.is_some()
            {
                return Err(anyhow!(
                    "stage-result manifest must leave `resource_metrics.memory_mb` and `resource_metrics.cpu_threads` empty when `resource_metrics.source` is `not_available`"
                ));
            }
        }
    }
    if manifest.outputs.is_empty() {
        return Err(anyhow!("stage-result manifest must declare at least one output in `outputs`"));
    }
    for output in &manifest.outputs {
        if output.artifact_id.trim().is_empty() {
            return Err(anyhow!(
                "stage-result manifest outputs must declare a non-empty `artifact_id`"
            ));
        }
        if output.declared_path.trim().is_empty() {
            return Err(anyhow!(
                "stage-result manifest outputs must declare a non-empty `declared_path`"
            ));
        }
        if output.realized_path.trim().is_empty() {
            return Err(anyhow!(
                "stage-result manifest outputs must declare a non-empty `realized_path`"
            ));
        }
        if output.role.trim().is_empty() {
            return Err(anyhow!("stage-result manifest outputs must declare a non-empty `role`"));
        }
    }
    Ok(())
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
