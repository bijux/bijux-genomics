use super::super::acquisition::EvidenceSet;
use super::super::declarations::EXPECTED_ACTIVE_ROW_CONSISTENCY_SURFACE_COUNT;
use super::super::inventory::BindingInventory;
use anyhow::{bail, Result};
pub(in crate::retained_tools) fn check_consistency(
    evidence: &EvidenceSet,
    bindings: &BindingInventory,
) -> Result<String> {
    if evidence.active_row_consistency.active_row_count != bindings.expected_retained_row_count
        || evidence.active_row_consistency.active_stage_count
            != bindings.expected_retained_stage_count
        || evidence.active_row_consistency.active_tool_count
            != bindings.expected_retained_tool_count
        || evidence.active_row_consistency.checked_surface_count
            != EXPECTED_ACTIVE_ROW_CONSISTENCY_SURFACE_COUNT
        || evidence.active_row_consistency.passed_surface_count
            != EXPECTED_ACTIVE_ROW_CONSISTENCY_SURFACE_COUNT
        || evidence.active_row_consistency.failed_surface_count != 0
        || !evidence.active_row_consistency.ok
    {
        bail!(
                    "BAM active-row consistency drifted: rows={}, stages={}, tools={}, checked_surfaces={}, passed_surfaces={}, failed_surfaces={}, ok={}",
                    evidence.active_row_consistency.active_row_count,
                    evidence.active_row_consistency.active_stage_count,
                    evidence.active_row_consistency.active_tool_count,
                    evidence.active_row_consistency.checked_surface_count,
                    evidence.active_row_consistency.passed_surface_count,
                    evidence.active_row_consistency.failed_surface_count,
                    evidence.active_row_consistency.ok
                );
    }
    Ok("validated active BAM rows against rendered commands, output declarations, expected results, parser fixtures, local jobs, and report-map rows".to_string())
}
