//! gl propagation admission bindings.
use crate::gl_propagation::model::GovernedVcfGlPropagationSmokeContract;
use crate::gl_propagation::model::GOVERNED_VCF_GL_PROPAGATION_ASSET_PROFILE_ID;
use crate::gl_propagation::model::GOVERNED_VCF_GL_PROPAGATION_CORPUS_ID;
use crate::gl_propagation::model::GOVERNED_VCF_GL_PROPAGATION_INPUT_FIXTURE_ID;
use crate::gl_propagation::model::GOVERNED_VCF_GL_PROPAGATION_SAMPLE_NAME;
use crate::gl_propagation::model::GOVERNED_VCF_GL_PROPAGATION_STAGE_ID;
use crate::gl_propagation::model::GOVERNED_VCF_GL_PROPAGATION_TOOL_ID;
use crate::inputs::GenotypeBenchmarkInputs;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

pub(crate) fn resolve_governed_vcf_gl_propagation_smoke_contract(
    inputs: &GenotypeBenchmarkInputs,
    requested_tool_id: &str,
) -> Result<GovernedVcfGlPropagationSmokeContract> {
    let matrix_row = inputs
        .rows()
        .find(|&row| row.stage_id == GOVERNED_VCF_GL_PROPAGATION_STAGE_ID)
        .cloned()
        .ok_or_else(|| {
            anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_GL_PROPAGATION_STAGE_ID}`")
        })?;
    if matrix_row.tool_id != GOVERNED_VCF_GL_PROPAGATION_TOOL_ID {
        bail!(
            "VCF gl_propagation smoke requires retained tool `{GOVERNED_VCF_GL_PROPAGATION_TOOL_ID}`, found `{}` in the governed matrix",
            matrix_row.tool_id
        );
    }
    if requested_tool_id != matrix_row.tool_id {
        bail!(
            "VCF gl_propagation smoke only retains tool `{}` for `{}`; requested `{requested_tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_GL_PROPAGATION_CORPUS_ID {
        bail!(
            "VCF gl_propagation smoke requires corpus `{GOVERNED_VCF_GL_PROPAGATION_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_GL_PROPAGATION_ASSET_PROFILE_ID {
        bail!(
            "VCF gl_propagation smoke requires asset profile `{GOVERNED_VCF_GL_PROPAGATION_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["gl_propagated_vcf".to_string()] {
        bail!(
            "VCF gl_propagation smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }

    Ok(GovernedVcfGlPropagationSmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_fixture_id: GOVERNED_VCF_GL_PROPAGATION_INPUT_FIXTURE_ID.to_string(),
        sample_name: GOVERNED_VCF_GL_PROPAGATION_SAMPLE_NAME.to_string(),
    })
}
