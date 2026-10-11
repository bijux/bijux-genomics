use super::model::{
    GovernedVcfStatsSmokeContract, GOVERNED_VCF_STATS_ASSET_PROFILE_ID,
    GOVERNED_VCF_STATS_CORPUS_ID, GOVERNED_VCF_STATS_INPUT_FIXTURE_ID,
    GOVERNED_VCF_STATS_SAMPLE_NAME, GOVERNED_VCF_STATS_STAGE_ID, GOVERNED_VCF_STATS_TOOL_ID,
};
use anyhow::{anyhow, bail, Result};
pub(super) fn resolve_governed_vcf_stats_smoke_contract(
    inputs: &crate::inputs::VariantBenchmarkInputs,
    requested_tool_id: &str,
) -> Result<GovernedVcfStatsSmokeContract> {
    let matrix_row = inputs
        .governed()
        .find(|row| row.stage_id == GOVERNED_VCF_STATS_STAGE_ID)
        .ok_or_else(|| anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_STATS_STAGE_ID}`"))?;
    if matrix_row.tool_id != GOVERNED_VCF_STATS_TOOL_ID {
        bail!(
            "VCF stats smoke requires retained tool `{GOVERNED_VCF_STATS_TOOL_ID}`, found `{}` in the governed matrix",
            matrix_row.tool_id
        );
    }
    if requested_tool_id != matrix_row.tool_id {
        bail!(
            "VCF stats smoke only retains tool `{}` for `{}`; requested `{requested_tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_STATS_CORPUS_ID {
        bail!(
            "VCF stats smoke requires corpus `{GOVERNED_VCF_STATS_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_STATS_ASSET_PROFILE_ID {
        bail!(
            "VCF stats smoke requires asset profile `{GOVERNED_VCF_STATS_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["stats_json".to_string()] {
        bail!(
            "VCF stats smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }

    Ok(GovernedVcfStatsSmokeContract {
        stage_id: matrix_row.stage_id.clone(),
        tool_id: matrix_row.tool_id.clone(),
        corpus_id: matrix_row.corpus_id.clone(),
        input_fixture_id: GOVERNED_VCF_STATS_INPUT_FIXTURE_ID.to_string(),
        sample_name: GOVERNED_VCF_STATS_SAMPLE_NAME.to_string(),
    })
}
