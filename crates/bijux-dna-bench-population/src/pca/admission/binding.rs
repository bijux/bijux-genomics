//! Owner: bijux-dna-bench-population
//! Pca admission binding.
use crate::inputs::PopulationBenchmarkInputs;
use crate::pca::model::{
    GovernedVcfPcaSmokeContract, GOVERNED_VCF_PCA_ASSET_PROFILE_ID, GOVERNED_VCF_PCA_CORPUS_ID,
    GOVERNED_VCF_PCA_INPUT_FIXTURE_ID, GOVERNED_VCF_PCA_STAGE_ID,
};
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

pub(crate) fn resolve_governed_vcf_pca_smoke_contract(
    inputs: &PopulationBenchmarkInputs,
    tool_id: &str,
) -> Result<GovernedVcfPcaSmokeContract> {
    let matrix_row = inputs
        .supported_rows()
        .iter()
        .find(|row| row.stage_id == GOVERNED_VCF_PCA_STAGE_ID && row.tool_id == tool_id)
        .ok_or_else(|| {
            anyhow!("VCF PCA smoke is missing a retained benchmark binding for `{tool_id}`")
        })?;
    if matrix_row.corpus_id != GOVERNED_VCF_PCA_CORPUS_ID {
        bail!(
            "VCF PCA smoke requires corpus `{GOVERNED_VCF_PCA_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_PCA_ASSET_PROFILE_ID {
        bail!(
            "VCF PCA smoke requires asset profile `{GOVERNED_VCF_PCA_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["pca_report".to_string()] {
        bail!(
            "VCF PCA smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }
    Ok(GovernedVcfPcaSmokeContract {
        stage: matrix_row.stage_id.clone(),
        tool: matrix_row.tool_id.clone(),
        corpus: matrix_row.corpus_id.clone(),
        input_fixture: GOVERNED_VCF_PCA_INPUT_FIXTURE_ID.to_string(),
    })
}
