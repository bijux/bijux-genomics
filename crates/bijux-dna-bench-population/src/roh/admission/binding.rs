//! Owner: bijux-dna-bench-population
//! Roh admission binding.
use crate::inputs::PopulationBenchmarkInputs;
use crate::roh::model::{
    GovernedVcfRohSmokeContract, GOVERNED_VCF_ROH_ASSET_PROFILE_ID, GOVERNED_VCF_ROH_CORPUS_ID,
    GOVERNED_VCF_ROH_INPUT_FIXTURE_ID, GOVERNED_VCF_ROH_STAGE_ID,
};
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

pub(crate) fn resolve_governed_vcf_roh_smoke_contract(
    inputs: &PopulationBenchmarkInputs,
    tool_id: &str,
) -> Result<GovernedVcfRohSmokeContract> {
    let matrix_row = inputs
        .governed_rows()
        .iter()
        .find(|row| row.stage_id == GOVERNED_VCF_ROH_STAGE_ID)
        .ok_or_else(|| anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_ROH_STAGE_ID}`"))?;
    if tool_id != matrix_row.tool_id {
        bail!(
            "VCF ROH smoke only retains tool `{}` for `{}`; requested `{tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_ROH_CORPUS_ID {
        bail!(
            "VCF ROH smoke requires corpus `{GOVERNED_VCF_ROH_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_ROH_ASSET_PROFILE_ID {
        bail!(
            "VCF ROH smoke requires asset profile `{GOVERNED_VCF_ROH_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["roh_report".to_string()] {
        bail!(
            "VCF ROH smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }
    Ok(GovernedVcfRohSmokeContract {
        stage: matrix_row.stage_id.clone(),
        tool: matrix_row.tool_id.clone(),
        corpus: matrix_row.corpus_id.clone(),
        input_fixture: GOVERNED_VCF_ROH_INPUT_FIXTURE_ID.to_string(),
    })
}
