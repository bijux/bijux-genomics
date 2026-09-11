use super::status::{ACTIVE_SCOPE_DETAIL, ACTIVE_SCOPE_PROOF_PATH, ACTIVE_SCOPE_STATE};
use anyhow::{anyhow, Result};
use bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow;
pub(super) fn require_governed_scope(rows: &[VcfActiveStageToolMatrixRow]) -> Result<()> {
    let expected_rows = [
        (
            "vcf.call",
            "bcftools",
            "production",
            "supported",
            ACTIVE_SCOPE_STATE,
            ACTIVE_SCOPE_DETAIL,
            ACTIVE_SCOPE_PROOF_PATH,
        ),
        (
            "vcf.impute",
            "beagle",
            "production",
            "supported",
            ACTIVE_SCOPE_STATE,
            ACTIVE_SCOPE_DETAIL,
            ACTIVE_SCOPE_PROOF_PATH,
        ),
        (
            "vcf.prepare_reference_panel",
            "bcftools",
            "production",
            "supported",
            ACTIVE_SCOPE_STATE,
            ACTIVE_SCOPE_DETAIL,
            ACTIVE_SCOPE_PROOF_PATH,
        ),
        (
            "vcf.phasing",
            "eagle",
            "production",
            "supported",
            ACTIVE_SCOPE_STATE,
            ACTIVE_SCOPE_DETAIL,
            ACTIVE_SCOPE_PROOF_PATH,
        ),
    ];

    for (
        stage_id,
        tool_id,
        tool_status,
        stage_support_status,
        scope_state,
        scope_detail,
        scope_proof_path,
    ) in expected_rows
    {
        let row = rows
            .iter()
            .find(|row| row.stage_id == stage_id && row.tool_id == tool_id)
            .ok_or_else(|| {
                anyhow!(
                    "VCF active-stage-tool matrix is missing retained binding `{stage_id}` / `{tool_id}`"
                )
            })?;
        if row.tool_status != tool_status
            || row.stage_support_status != stage_support_status
            || row.scope_state != scope_state
            || row.scope_detail != scope_detail
            || row.scope_proof_path != scope_proof_path
        {
            return Err(anyhow!(
                "VCF active-stage-tool matrix drifted for `{stage_id}` / `{tool_id}`"
            ));
        }
    }

    Ok(())
}
