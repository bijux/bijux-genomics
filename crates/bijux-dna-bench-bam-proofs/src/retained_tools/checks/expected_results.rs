use super::super::bindings::ensure_binding_subset;
use super::super::inventory::BindingInventory;
use anyhow::{bail, Result};
pub(in crate::retained_tools) fn check_expected_results(
    bindings: &BindingInventory,
) -> Result<String> {
    if bindings.expected_result_bindings.len() != bindings.expected_retained_row_count {
        bail!(
            "BAM expected results drifted: bam_rows={} expected={}",
            bindings.expected_result_bindings.len(),
            bindings.expected_retained_row_count
        );
    }
    ensure_binding_subset(
        "bam retained expected results",
        "expected benchmark results",
        &bindings.retained_bindings,
        &bindings.expected_result_bindings,
    )?;
    Ok("validated one governed expected-result row for every retained BAM binding".to_string())
}
