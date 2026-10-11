//! Validation for persisted local benchmark stage-completion reports.

use std::collections::BTreeSet;

use crate::{
    BenchError, BenchmarkLocalStageManifestCompletionReport,
    BenchmarkLocalStageOutputCompletionReport, BENCHMARK_LOCAL_STAGE_MANIFEST_COMPLETION_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_OUTPUT_COMPLETION_SCHEMA_V1,
};

/// Validate a local stage-manifest completion report.
///
/// # Errors
/// Returns an error when identity, cardinality, or completion invariants are invalid.
pub fn validate_benchmark_local_stage_manifest_completion(
    report: &BenchmarkLocalStageManifestCompletionReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_STAGE_MANIFEST_COMPLETION_SCHEMA_V1 {
        return invalid(format!(
            "unsupported local stage-manifest completion schema `{}`",
            report.schema_version
        ));
    }
    validate_report_identity(
        &report.fake_run_root,
        &report.report_output_path,
        &report.source_stage_command_manifest_path,
    )?;
    validate_report_counts(
        report.stage_count,
        report.complete_stage_count,
        report.incomplete_stage_count,
        report.complete,
        report.stages.len(),
    )?;
    let mut stage_ids = BTreeSet::new();
    for stage in &report.stages {
        require_text(&stage.stage_id, "stages[].stage_id")?;
        require_text(&stage.tool_id, "stages[].tool_id")?;
        require_text(&stage.stage_manifest_path, "stages[].stage_manifest_path")?;
        if !stage_ids.insert(stage.stage_id.as_str()) {
            return invalid(format!(
                "stage-manifest completion repeats stage `{}`",
                stage.stage_id
            ));
        }
        if stage.present_output_count > stage.declared_output_count {
            return invalid("stages[].present_output_count exceeds declared_output_count");
        }
        if stage.complete != stage.manifest_exists {
            return invalid("stages[].complete must match manifest_exists");
        }
    }
    let observed_complete = report.stages.iter().filter(|stage| stage.complete).count();
    if observed_complete != report.complete_stage_count {
        return invalid("complete_stage_count does not match stages[].complete");
    }
    Ok(())
}

/// Validate a local stage-output completion report.
///
/// # Errors
/// Returns an error when identity, cardinality, or missing-output invariants are invalid.
pub fn validate_benchmark_local_stage_output_completion(
    report: &BenchmarkLocalStageOutputCompletionReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_STAGE_OUTPUT_COMPLETION_SCHEMA_V1 {
        return invalid(format!(
            "unsupported local stage-output completion schema `{}`",
            report.schema_version
        ));
    }
    validate_report_identity(
        &report.fake_run_root,
        &report.report_output_path,
        &report.source_stage_command_manifest_path,
    )?;
    validate_report_counts(
        report.stage_count,
        report.complete_stage_count,
        report.incomplete_stage_count,
        report.complete,
        report.stages.len(),
    )?;
    let mut stage_ids = BTreeSet::new();
    for stage in &report.stages {
        require_text(&stage.stage_id, "stages[].stage_id")?;
        require_text(&stage.tool_id, "stages[].tool_id")?;
        if !stage_ids.insert(stage.stage_id.as_str()) {
            return invalid(format!("stage-output completion repeats stage `{}`", stage.stage_id));
        }
        if stage.missing_output_count != stage.missing_outputs.len()
            || stage.declared_output_count
                != stage.present_output_count.saturating_add(stage.missing_output_count)
        {
            return invalid("stage output counts are inconsistent");
        }
        if stage.complete != (stage.missing_output_count == 0) {
            return invalid("stages[].complete must match missing_output_count");
        }
        let mut artifact_ids = BTreeSet::new();
        for missing in &stage.missing_outputs {
            require_text(&missing.artifact_id, "stages[].missing_outputs[].artifact_id")?;
            require_text(&missing.declared_path, "stages[].missing_outputs[].declared_path")?;
            require_text(
                &missing.expected_fake_run_path,
                "stages[].missing_outputs[].expected_fake_run_path",
            )?;
            require_text(&missing.role, "stages[].missing_outputs[].role")?;
            if !artifact_ids.insert(missing.artifact_id.as_str()) {
                return invalid(format!(
                    "stage `{}` repeats missing artifact `{}`",
                    stage.stage_id, missing.artifact_id
                ));
            }
        }
    }
    let observed_complete = report.stages.iter().filter(|stage| stage.complete).count();
    if observed_complete != report.complete_stage_count {
        return invalid("complete_stage_count does not match stages[].complete");
    }
    Ok(())
}

fn validate_report_identity(
    fake_run_root: &str,
    report_output_path: &str,
    source_stage_command_manifest_path: &str,
) -> Result<(), BenchError> {
    require_text(fake_run_root, "fake_run_root")?;
    require_text(report_output_path, "report_output_path")?;
    require_text(source_stage_command_manifest_path, "source_stage_command_manifest_path")
}

fn validate_report_counts(
    stage_count: usize,
    complete_stage_count: usize,
    incomplete_stage_count: usize,
    complete: bool,
    observed_stage_count: usize,
) -> Result<(), BenchError> {
    if stage_count == 0 || stage_count != observed_stage_count {
        return invalid("stage_count must equal a non-empty stages array");
    }
    if complete_stage_count.saturating_add(incomplete_stage_count) != stage_count {
        return invalid("complete and incomplete stage counts must sum to stage_count");
    }
    if complete != (incomplete_stage_count == 0) {
        return invalid("report complete must match incomplete_stage_count");
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local stage-completion report requires non-empty {field}"));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> Result<(), BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
