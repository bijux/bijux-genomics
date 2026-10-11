use super::super::acquisition::EvidenceSet;
use super::super::bindings::ensure_binding_subset;
use super::super::inventory::BindingInventory;
use anyhow::{bail, Result};
pub(in crate::retained_tools) fn check_report_map(
    evidence: &EvidenceSet,
    bindings: &BindingInventory,
) -> Result<String> {
    if evidence.report_map.expected_result_row_count != bindings.expected_retained_row_count
        || evidence.report_map.row_count != bindings.expected_retained_row_count
        || evidence.report_map.stage_count != bindings.expected_retained_stage_count
        || evidence.report_map.tool_count != bindings.expected_retained_tool_count
    {
        bail!(
                    "BAM report map drifted: evidence.expected_results={}, evidence.report_map={}, stages={}, tools={}",
                    evidence.report_map.expected_result_row_count,
                    evidence.report_map.row_count,
                    evidence.report_map.stage_count,
                    evidence.report_map.tool_count
                );
    }
    ensure_binding_subset(
        "bam retained report map",
        "report map coverage",
        &bindings.retained_bindings,
        &bindings.report_map_bindings,
    )?;
    Ok("validated report-map placement for every retained BAM expected-result row".to_string())
}
