//! Validation for persisted local benchmark runtime metrics.

use std::collections::BTreeSet;

use crate::{
    BenchError, BenchmarkLocalStageRuntimeMetricsReport, BENCHMARK_LOCAL_RUNTIME_METRICS_SCHEMA_V1,
};

/// Validate one local runtime-metrics report.
///
/// # Errors
/// Returns an error when identity, cardinality, or runtime invariants are invalid.
pub fn validate_benchmark_local_runtime_metrics(
    report: &BenchmarkLocalStageRuntimeMetricsReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_RUNTIME_METRICS_SCHEMA_V1 {
        return invalid(format!(
            "unsupported local runtime-metrics schema `{}`",
            report.schema_version
        ));
    }
    require_text(&report.fake_run_root, "fake_run_root")?;
    require_text(&report.report_output_path, "report_output_path")?;
    require_text(&report.source_stage_command_manifest_path, "source_stage_command_manifest_path")?;
    if report.stage_count == 0 || report.stage_count != report.stages.len() {
        return invalid("local runtime-metrics stage_count must equal a non-empty stages array");
    }
    let mut stage_ids = BTreeSet::new();
    for stage in &report.stages {
        require_text(&stage.stage_id, "stages[].stage_id")?;
        require_text(&stage.tool_id, "stages[].tool_id")?;
        require_text(&stage.manifest_path, "stages[].manifest_path")?;
        require_text(&stage.runtime_mode, "stages[].runtime_mode")?;
        require_text(&stage.started_at, "stages[].started_at")?;
        require_text(&stage.finished_at, "stages[].finished_at")?;
        if !stage.elapsed_seconds.is_finite() || stage.elapsed_seconds.is_sign_negative() {
            return invalid("stages[].elapsed_seconds must be finite and non-negative");
        }
        if !stage_ids.insert(stage.stage_id.as_str()) {
            return invalid(format!(
                "local runtime-metrics report repeats stage `{}`",
                stage.stage_id
            ));
        }
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local runtime-metrics report requires non-empty {field}"));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> Result<(), BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
