//! postprocess admission bindings.
use crate::inputs::VariantBenchmarkInputs;
use crate::postprocess::model::GovernedVcfPostprocessSmokeContract;
use crate::postprocess::model::GOVERNED_VCF_POSTPROCESS_ASSET_PROFILE_ID;
use crate::postprocess::model::GOVERNED_VCF_POSTPROCESS_CORPUS_ID;
use crate::postprocess::model::GOVERNED_VCF_POSTPROCESS_INPUT_FIXTURE_ID;
use crate::postprocess::model::GOVERNED_VCF_POSTPROCESS_STAGE_ID;
use crate::postprocess::model::GOVERNED_VCF_POSTPROCESS_TOOL_ID;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

pub(crate) fn resolve_governed_vcf_postprocess_smoke_contract(
    inputs: &VariantBenchmarkInputs,
    requested_tool_id: &str,
) -> Result<GovernedVcfPostprocessSmokeContract> {
    let matrix_row = inputs
        .governed()
        .find(|&row| row.stage_id == GOVERNED_VCF_POSTPROCESS_STAGE_ID)
        .cloned()
        .ok_or_else(|| {
            anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_POSTPROCESS_STAGE_ID}`")
        })?;
    if matrix_row.tool_id != GOVERNED_VCF_POSTPROCESS_TOOL_ID {
        bail!(
            "VCF postprocess smoke requires retained tool `{GOVERNED_VCF_POSTPROCESS_TOOL_ID}`, found `{}` in the governed matrix",
            matrix_row.tool_id
        );
    }
    if requested_tool_id != matrix_row.tool_id {
        bail!(
            "VCF postprocess smoke only retains tool `{}` for `{}`; requested `{requested_tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_POSTPROCESS_CORPUS_ID {
        bail!(
            "VCF postprocess smoke requires corpus `{GOVERNED_VCF_POSTPROCESS_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_POSTPROCESS_ASSET_PROFILE_ID {
        bail!(
            "VCF postprocess smoke requires asset profile `{GOVERNED_VCF_POSTPROCESS_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["postprocess_vcf".to_string()] {
        bail!(
            "VCF postprocess smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }
    Ok(GovernedVcfPostprocessSmokeContract {
        stage: matrix_row.stage_id,
        tool: matrix_row.tool_id,
        corpus: matrix_row.corpus_id,
        input_fixture: GOVERNED_VCF_POSTPROCESS_INPUT_FIXTURE_ID.to_string(),
    })
}
