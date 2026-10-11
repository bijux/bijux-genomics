use super::super::acquisition::EvidenceSet;
use super::super::bindings::ensure_binding_subset;
use super::super::inventory::BindingInventory;
use anyhow::{bail, Result};
pub(in crate::retained_tools) fn check_outputs(
    evidence: &EvidenceSet,
    bindings: &BindingInventory,
) -> Result<String> {
    if evidence.output_declarations.row_count != bindings.expected_retained_row_count
        || evidence.output_declarations.adapter_row_count != bindings.expected_retained_row_count
        || evidence.output_declarations.complete_adapter_row_count
            != bindings.expected_retained_row_count
        || evidence.output_declarations.incomplete_adapter_row_count != 0
        || evidence.output_declarations.missing_adapter_row_count != 0
    {
        bail!(
                    "BAM output declarations drifted: rows={}, adapter_rows={}, complete={}, incomplete={}, missing_adapter={}",
                    evidence.output_declarations.row_count,
                    evidence.output_declarations.adapter_row_count,
                    evidence.output_declarations.complete_adapter_row_count,
                    evidence.output_declarations.incomplete_adapter_row_count,
                    evidence.output_declarations.missing_adapter_row_count
                );
    }
    ensure_binding_subset(
        "bam retained output declarations",
        "output declaration coverage",
        &bindings.retained_bindings,
        &bindings.output_declaration_bindings,
    )?;
    Ok("validated declared raw outputs, normalized metrics, stdout/stderr, and manifests for every retained BAM binding".to_string())
}
