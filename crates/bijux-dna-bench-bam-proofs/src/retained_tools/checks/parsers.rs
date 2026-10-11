use super::super::acquisition::EvidenceSet;
use super::super::bindings::ensure_binding_subset;
use super::super::inventory::BindingInventory;
use anyhow::{bail, Result};
pub(in crate::retained_tools) fn check_parsers(
    evidence: &EvidenceSet,
    bindings: &BindingInventory,
) -> Result<String> {
    if evidence.parser_coverage.row_count != bindings.expected_retained_row_count
        || evidence.parser_coverage.stage_count != bindings.expected_retained_stage_count
        || evidence.parser_coverage.tool_count != bindings.expected_retained_tool_count
        || evidence.parser_coverage.covered_row_count != bindings.expected_retained_row_count
        || evidence.parser_coverage.missing_row_count != 0
        || (evidence.parser_coverage.parser_fixture_coverage_percent - 100.0).abs() > f64::EPSILON
    {
        bail!(
                    "BAM parser fixture coverage drifted: rows={}, stages={}, tools={}, covered={}, missing={}, percent={}",
                    evidence.parser_coverage.row_count,
                    evidence.parser_coverage.stage_count,
                    evidence.parser_coverage.tool_count,
                    evidence.parser_coverage.covered_row_count,
                    evidence.parser_coverage.missing_row_count,
                    evidence.parser_coverage.parser_fixture_coverage_percent
                );
    }
    ensure_binding_subset(
        "bam retained parser fixture coverage",
        "parser fixture coverage",
        &bindings.retained_bindings,
        &bindings.parser_bindings,
    )?;
    Ok("validated 100% parser fixture coverage for the retained BAM binding slice".to_string())
}
