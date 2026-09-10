//! Owner: bijux-dna-bench-population
//! Admixture admission binding.
use crate::admixture::model::{
    GovernedVcfAdmixtureSmokeContract, GOVERNED_VCF_ADMIXTURE_ASSET_PROFILE_ID,
    GOVERNED_VCF_ADMIXTURE_CORPUS_ID, GOVERNED_VCF_ADMIXTURE_INPUT_FIXTURE_ID,
    GOVERNED_VCF_ADMIXTURE_STAGE_ID,
};
use crate::inputs::PopulationBenchmarkInputs;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

pub(crate) fn resolve_governed_vcf_admixture_smoke_contract(
    inputs: &PopulationBenchmarkInputs,
    tool_id: &str,
) -> Result<GovernedVcfAdmixtureSmokeContract> {
    let matrix_row = inputs.supported_rows().iter()
        .find(|row| row.stage_id == GOVERNED_VCF_ADMIXTURE_STAGE_ID && row.tool_id == tool_id)
        .ok_or_else(|| {
            anyhow!(
                "VCF admixture smoke has no governed binding for `{GOVERNED_VCF_ADMIXTURE_STAGE_ID}` / `{tool_id}`"
            )
        })?;
    if matrix_row.corpus_id != GOVERNED_VCF_ADMIXTURE_CORPUS_ID {
        bail!(
            "VCF admixture smoke requires corpus `{GOVERNED_VCF_ADMIXTURE_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_ADMIXTURE_ASSET_PROFILE_ID {
        bail!(
            "VCF admixture smoke requires asset profile `{GOVERNED_VCF_ADMIXTURE_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["admixture_report".to_string()] {
        bail!(
            "VCF admixture smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }
    Ok(GovernedVcfAdmixtureSmokeContract {
        stage: matrix_row.stage_id.clone(),
        tool: matrix_row.tool_id.clone(),
        corpus: matrix_row.corpus_id.clone(),
        input_fixture: GOVERNED_VCF_ADMIXTURE_INPUT_FIXTURE_ID.to_string(),
    })
}
