//! Owner: bijux-dna-bench-model
//! Semantic admission for stage/tool benchmark-readiness reports.

use std::collections::BTreeSet;

use crate::{
    model::{StageToolBenchmarkReadyReport, StageToolBenchmarkReadySurfaceStatus},
    BenchError,
};

use super::STAGE_TOOL_BENCHMARK_READY_SCHEMA_V1;

/// Validate the persisted stage/tool benchmark-readiness report.
///
/// # Errors
/// Returns an error when identities, aggregate counts, or gate state do not close.
pub fn validate_stage_tool_benchmark_ready_report(
    report: &StageToolBenchmarkReadyReport,
) -> Result<(), BenchError> {
    if report.schema_version != STAGE_TOOL_BENCHMARK_READY_SCHEMA_V1 {
        return invalid("unsupported stage/tool benchmark-readiness schema");
    }
    require_text(&report.output_path, "output_path")?;
    if report.benchmark_ready_pair_count.checked_add(report.excluded_pair_count)
        != Some(report.expected_pair_count)
    {
        return invalid("stage/tool expected-pair aggregate does not close");
    }
    if report.failing_pair_count != report.failing_pairs.len()
        || report.excluded_pair_count != report.excluded_pairs.len()
    {
        return invalid("stage/tool persisted row aggregates do not close");
    }
    if report.passes_gate != report.failing_pairs.is_empty() {
        return invalid("stage/tool gate state disagrees with failing pairs");
    }
    if report.surface_summaries.is_empty() {
        return invalid("stage/tool report has no surface summaries");
    }

    let mut surface_ids = BTreeSet::new();
    for surface in &report.surface_summaries {
        require_text(&surface.surface_id, "surface_summaries[].surface_id")?;
        require_text(&surface.measured_scope, "surface_summaries[].measured_scope")?;
        require_text(&surface.detail, "surface_summaries[].detail")?;
        if !surface_ids.insert(surface.surface_id.as_str()) {
            return invalid("stage/tool report repeats a surface identity");
        }
        if surface.evidence_paths.is_empty()
            || surface.evidence_paths.iter().any(|path| path.trim().is_empty())
        {
            return invalid("stage/tool surface has no concrete evidence path");
        }
        if surface.surface_status == StageToolBenchmarkReadySurfaceStatus::ReadySliceComplete
            && surface.failing_count != 0
        {
            return invalid("ready stage/tool surface declares failing rows");
        }
        if surface.surface_status == StageToolBenchmarkReadySurfaceStatus::FailingPairsPresent
            && surface.failing_count == 0
        {
            return invalid("failing stage/tool surface has no failing rows");
        }
    }

    let mut pair_ids = BTreeSet::new();
    for row in &report.failing_pairs {
        validate_pair_identity(&row.row_id, &row.domain, &row.stage_id, &row.tool_id)?;
        if !pair_ids.insert(row.row_id.as_str()) {
            return invalid("stage/tool report repeats a pair identity");
        }
        if row.failure_surfaces.is_empty()
            || row.failure_surfaces.iter().any(|surface| !surface_ids.contains(surface.as_str()))
        {
            return invalid("failing stage/tool pair references an unknown failure surface");
        }
        require_text(&row.reason, "failing_pairs[].reason")?;
    }
    for row in &report.excluded_pairs {
        validate_pair_identity(&row.row_id, &row.domain, &row.stage_id, &row.tool_id)?;
        if !pair_ids.insert(row.row_id.as_str()) {
            return invalid("stage/tool report repeats a pair identity");
        }
        require_text(&row.reason, "excluded_pairs[].reason")?;
    }
    Ok(())
}

fn validate_pair_identity(
    row_id: &str,
    domain: &str,
    stage_id: &str,
    tool_id: &str,
) -> Result<(), BenchError> {
    for (value, field) in [(domain, "domain"), (stage_id, "stage_id"), (tool_id, "tool_id")] {
        require_text(value, field)?;
    }
    if row_id != format!("{domain}:{stage_id}:{tool_id}") {
        return invalid("stage/tool row identity does not match its coordinates");
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(&format!("{field} must not be empty"));
    }
    Ok(())
}

fn invalid(message: &str) -> Result<(), BenchError> {
    Err(BenchError::InvalidPolicy(message.to_string()))
}
