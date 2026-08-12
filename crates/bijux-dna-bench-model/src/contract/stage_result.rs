//! Validation for persisted benchmark stage results.

use crate::diagnostics::BenchError;
use crate::model::{
    BenchmarkStageResultManifest, BenchmarkStageResultResourceMetricSource,
    BenchmarkStageResultValidationReport,
};

use super::{BENCHMARK_STAGE_RESULT_SCHEMA_V2, BENCHMARK_STAGE_RESULT_VALIDATION_SCHEMA_V1};

/// Validate one canonical stage-result manifest.
///
/// # Errors
/// Returns an error when identity, runtime, resource, or output invariants are violated.
pub fn validate_benchmark_stage_result_manifest(
    manifest: &BenchmarkStageResultManifest,
) -> Result<(), BenchError> {
    if manifest.schema_version != BENCHMARK_STAGE_RESULT_SCHEMA_V2 {
        return invalid(format!("unsupported stage-result schema `{}`", manifest.schema_version));
    }
    require_text(&manifest.stage_id, "stage_id")?;
    require_text(&manifest.tool.id, "tool.id")?;
    require_text(&manifest.command.rendered, "command.rendered")?;
    require_text(&manifest.runtime.mode, "runtime.mode")?;
    require_text(&manifest.runtime.started_at, "runtime.started_at")?;
    require_text(&manifest.runtime.finished_at, "runtime.finished_at")?;
    if !manifest.runtime.elapsed_seconds.is_finite()
        || manifest.runtime.elapsed_seconds.is_sign_negative()
    {
        return invalid("runtime.elapsed_seconds must be finite and non-negative");
    }
    if manifest
        .resource_metrics
        .memory_mb
        .is_some_and(|value| !value.is_finite() || value.is_sign_negative())
    {
        return invalid("resource_metrics.memory_mb must be finite and non-negative");
    }
    match manifest.resource_metrics.source {
        BenchmarkStageResultResourceMetricSource::Measured
        | BenchmarkStageResultResourceMetricSource::Estimated => {
            if manifest.resource_metrics.memory_mb.is_none()
                && manifest.resource_metrics.cpu_threads.is_none()
            {
                return invalid(
                    "stage-result manifest must declare at least one resource metric when resource metrics are measured or estimated",
                );
            }
        }
        BenchmarkStageResultResourceMetricSource::NotAvailable => {
            if manifest.resource_metrics.memory_mb.is_some()
                || manifest.resource_metrics.cpu_threads.is_some()
            {
                return invalid(
                    "stage-result manifest must leave `resource_metrics.memory_mb` and `resource_metrics.cpu_threads` empty when `resource_metrics.source` is `not_available`",
                );
            }
        }
    }
    if manifest.outputs.is_empty() {
        return invalid("outputs must contain at least one artifact");
    }
    for output in &manifest.outputs {
        require_text(&output.artifact_id, "outputs[].artifact_id")?;
        require_text(&output.declared_path, "outputs[].declared_path")?;
        require_text(&output.realized_path, "outputs[].realized_path")?;
        require_text(&output.role, "outputs[].role")?;
    }
    Ok(())
}

/// Validate one persisted stage-result validation report.
///
/// # Errors
/// Returns an error when the report is not the exact successful projection of a valid manifest.
pub fn validate_benchmark_stage_result_validation_report(
    report: &BenchmarkStageResultValidationReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_STAGE_RESULT_VALIDATION_SCHEMA_V1 {
        return invalid(format!(
            "unsupported stage-result validation schema `{}`",
            report.schema_version
        ));
    }
    require_text(&report.manifest_path, "manifest_path")?;
    require_text(&report.stage_id, "stage_id")?;
    require_text(&report.tool_id, "tool_id")?;
    if report.output_count == 0 {
        return invalid("stage-result validation report requires at least one output");
    }
    if !report.valid {
        return invalid("stage-result validation report cannot claim an invalid manifest");
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("stage-result manifest requires non-empty {field}"));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> Result<(), BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
