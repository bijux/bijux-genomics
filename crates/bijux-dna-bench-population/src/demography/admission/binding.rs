//! Owner: bijux-dna-bench-population
//! Demography admission binding.
use crate::demography::model::{
    GovernedVcfDemographySmokeContract, GOVERNED_VCF_DEMOGRAPHY_ASSET_PROFILE_ID,
    GOVERNED_VCF_DEMOGRAPHY_CORPUS_ID, GOVERNED_VCF_DEMOGRAPHY_EXPECTED_OUTPUT_ID,
    GOVERNED_VCF_DEMOGRAPHY_INPUT_FIXTURE_ID, GOVERNED_VCF_DEMOGRAPHY_STAGE_ID,
};
use crate::inputs::PopulationBenchmarkInputs;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

pub(crate) fn resolve_governed_vcf_demography_smoke_contract(
    inputs: &PopulationBenchmarkInputs,
    tool_id: &str,
) -> Result<GovernedVcfDemographySmokeContract> {
    let matrix_row = inputs
        .governed_rows()
        .iter()
        .find(|row| row.stage_id == GOVERNED_VCF_DEMOGRAPHY_STAGE_ID)
        .ok_or_else(|| {
            anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_DEMOGRAPHY_STAGE_ID}`")
        })?;
    if matrix_row.tool_id != tool_id {
        bail!(
            "VCF demography smoke only retains tool `{}`; requested `{tool_id}`",
            matrix_row.tool_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_DEMOGRAPHY_CORPUS_ID {
        bail!(
            "VCF demography smoke requires corpus `{GOVERNED_VCF_DEMOGRAPHY_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_DEMOGRAPHY_ASSET_PROFILE_ID {
        bail!(
            "VCF demography smoke requires asset profile `{GOVERNED_VCF_DEMOGRAPHY_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec![GOVERNED_VCF_DEMOGRAPHY_EXPECTED_OUTPUT_ID.to_string()] {
        bail!(
            "VCF demography smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }

    Ok(GovernedVcfDemographySmokeContract {
        stage: matrix_row.stage_id.clone(),
        tool: matrix_row.tool_id.clone(),
        corpus: matrix_row.corpus_id.clone(),
        input_fixture: GOVERNED_VCF_DEMOGRAPHY_INPUT_FIXTURE_ID.to_string(),
    })
}
