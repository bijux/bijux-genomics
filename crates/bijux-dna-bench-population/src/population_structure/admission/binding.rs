//! Owner: bijux-dna-bench-population
//! Population structure admission binding.
use crate::inputs::PopulationBenchmarkInputs;
use crate::population_structure::model::{
    GovernedVcfPopulationStructureSmokeContract,
    GOVERNED_VCF_POPULATION_STRUCTURE_ASSET_PROFILE_ID,
    GOVERNED_VCF_POPULATION_STRUCTURE_CORPUS_ID,
    GOVERNED_VCF_POPULATION_STRUCTURE_INPUT_FIXTURE_ID, GOVERNED_VCF_POPULATION_STRUCTURE_STAGE_ID,
};
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

pub(crate) fn resolve_governed_vcf_population_structure_smoke_contract(
    inputs: &PopulationBenchmarkInputs,
    tool_id: &str,
) -> Result<GovernedVcfPopulationStructureSmokeContract> {
    let matrix_row = inputs.supported_rows().iter()
        .find(|row| {
            row.stage_id == GOVERNED_VCF_POPULATION_STRUCTURE_STAGE_ID && row.tool_id == tool_id
        })
        .ok_or_else(|| {
            anyhow!(
                "VCF population-structure smoke has no governed binding for `{GOVERNED_VCF_POPULATION_STRUCTURE_STAGE_ID}` / `{tool_id}`"
            )
        })?;
    if matrix_row.corpus_id != GOVERNED_VCF_POPULATION_STRUCTURE_CORPUS_ID {
        bail!(
            "VCF population-structure smoke requires corpus `{GOVERNED_VCF_POPULATION_STRUCTURE_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_POPULATION_STRUCTURE_ASSET_PROFILE_ID {
        bail!(
            "VCF population-structure smoke requires asset profile `{GOVERNED_VCF_POPULATION_STRUCTURE_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["population_structure_report".to_string()] {
        bail!(
            "VCF population-structure smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }
    let (pca_tool_id, admixture_tool_id, normalization_pca_tool_id) = match tool_id {
        "eigensoft" => ("eigensoft", "plink2", "eigensoft"),
        "plink" => ("plink", "plink", "plink2"),
        "plink2" => ("plink2", "plink2", "plink2"),
        other => bail!("unsupported population-structure tool binding `{other}`"),
    };
    Ok(GovernedVcfPopulationStructureSmokeContract {
        stage: matrix_row.stage_id.clone(),
        tool: matrix_row.tool_id.clone(),
        pca_tool_id: pca_tool_id.to_string(),
        admixture_tool_id: admixture_tool_id.to_string(),
        normalization_pca_tool_id: normalization_pca_tool_id.to_string(),
        corpus: matrix_row.corpus_id.clone(),
        input_fixture: GOVERNED_VCF_POPULATION_STRUCTURE_INPUT_FIXTURE_ID.to_string(),
    })
}
