//! filter admission bindings.
use crate::filter::model::GovernedVcfFilterSmokeContract;
use crate::filter::model::GOVERNED_VCF_FILTER_ASSET_PROFILE_ID;
use crate::filter::model::GOVERNED_VCF_FILTER_CORPUS_ID;
use crate::filter::model::GOVERNED_VCF_FILTER_INPUT_FIXTURE_ID;
use crate::filter::model::GOVERNED_VCF_FILTER_SAMPLE_NAME;
use crate::filter::model::GOVERNED_VCF_FILTER_STAGE_ID;
use crate::filter::model::GOVERNED_VCF_FILTER_TOOL_ID;
use crate::inputs::VariantBenchmarkInputs;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

pub(crate) fn resolve_governed_vcf_filter_smoke_contract(
    inputs: &VariantBenchmarkInputs,
    requested_tool_id: &str,
) -> Result<GovernedVcfFilterSmokeContract> {
    let matrix_row = inputs
        .governed()
        .find(|&row| row.stage_id == GOVERNED_VCF_FILTER_STAGE_ID)
        .cloned()
        .ok_or_else(|| anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_FILTER_STAGE_ID}`"))?;
    if matrix_row.tool_id != GOVERNED_VCF_FILTER_TOOL_ID {
        bail!(
            "VCF filter smoke requires retained tool `{GOVERNED_VCF_FILTER_TOOL_ID}`, found `{}` in the governed matrix",
            matrix_row.tool_id
        );
    }
    if requested_tool_id != matrix_row.tool_id {
        bail!(
            "VCF filter smoke only retains tool `{}` for `{}`; requested `{requested_tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_FILTER_CORPUS_ID {
        bail!(
            "VCF filter smoke requires corpus `{GOVERNED_VCF_FILTER_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_FILTER_ASSET_PROFILE_ID {
        bail!(
            "VCF filter smoke requires asset profile `{GOVERNED_VCF_FILTER_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["filtered_vcf".to_string()] {
        bail!(
            "VCF filter smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }

    Ok(GovernedVcfFilterSmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_fixture_id: GOVERNED_VCF_FILTER_INPUT_FIXTURE_ID.to_string(),
        sample_name: GOVERNED_VCF_FILTER_SAMPLE_NAME.to_string(),
    })
}
