use super::super::acquisition::EvidenceSet;
use super::super::bindings::ensure_binding_subset;
use super::super::inventory::BindingInventory;
use anyhow::{bail, Result};
pub(in crate::retained_tools) fn check_runtime(
    evidence: &EvidenceSet,
    bindings: &BindingInventory,
) -> Result<String> {
    if evidence.local_smoke.row_count != bindings.expected_retained_row_count
        || evidence.local_smoke.stage_count != bindings.expected_retained_stage_count
        || evidence.local_smoke.tool_count != bindings.expected_retained_tool_count
        || evidence.local_smoke.host_stage_smoke_row_count
            + evidence.local_smoke.container_smoke_row_count
            + evidence.local_smoke.external_artifact_row_count
            != bindings.expected_retained_row_count
    {
        bail!(
                    "BAM local/container smoke drifted: rows={}, stages={}, tools={}, host_stage_smokes={}, container_smokes={}, external_artifacts={}",
                    evidence.local_smoke.row_count,
                    evidence.local_smoke.stage_count,
                    evidence.local_smoke.tool_count,
                    evidence.local_smoke.host_stage_smoke_row_count,
                    evidence.local_smoke.container_smoke_row_count,
                    evidence.local_smoke.external_artifact_row_count
                );
    }
    ensure_binding_subset(
        "bam retained local/container smoke",
        "local/container smoke coverage",
        &bindings.retained_bindings,
        &bindings.local_smoke_bindings,
    )?;
    Ok(format!(
                "validated runtime smoke coverage for every retained BAM binding with {} host row(s), {} container row(s), and {} external-artifact admission row(s)",
                evidence.local_smoke.host_stage_smoke_row_count,
                evidence.local_smoke.container_smoke_row_count,
                evidence.local_smoke.external_artifact_row_count
            ))
}
