use super::status::{
    count_scope_state, ACTIVE_SCOPE_STATE, COMPLETE_SCOPE_STATE, REMOVED_FROM_SCOPE_STATE,
};
use anyhow::{anyhow, Result};
use bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow;
use std::collections::BTreeSet;
pub(super) fn ensure_vcf_active_stage_tool_matrix_contract(
    rows: &[VcfActiveStageToolMatrixRow],
) -> Result<()> {
    let unique_rows = rows
        .iter()
        .map(|row| (row.stage_id.as_str(), row.tool_id.as_str()))
        .collect::<BTreeSet<_>>();
    if unique_rows.len() != rows.len() {
        return Err(anyhow!(
            "VCF active-stage-tool matrix must keep one row per retained VCF stage/tool binding"
        ));
    }

    if rows.iter().any(|row| {
        row.scope_state != ACTIVE_SCOPE_STATE
            && row.scope_state != COMPLETE_SCOPE_STATE
            && row.scope_state != REMOVED_FROM_SCOPE_STATE
    }) {
        return Err(anyhow!(
            "VCF active-stage-tool matrix encountered a row outside active/complete/removed scope resolution"
        ));
    }

    if rows.iter().any(|row| !matches!(row.tool_status.as_str(), "production" | "supported")) {
        return Err(anyhow!(
            "VCF active-stage-tool matrix requires a production or supported domain identity for every retained binding"
        ));
    }

    let active_row_count = count_scope_state(rows, ACTIVE_SCOPE_STATE);
    let complete_row_count = count_scope_state(rows, COMPLETE_SCOPE_STATE);
    let removed_row_count = count_scope_state(rows, REMOVED_FROM_SCOPE_STATE);
    if active_row_count + complete_row_count + removed_row_count != rows.len() {
        return Err(anyhow!(
            "VCF active-stage-tool matrix must classify every retained binding into exactly one scope state"
        ));
    }

    super::governed::require_governed_scope(rows)?;

    Ok(())
}
