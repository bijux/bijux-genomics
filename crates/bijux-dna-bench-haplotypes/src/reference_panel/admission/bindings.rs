//! reference panel admission bindings.
use crate::inputs::HaplotypeBenchmarkInputs;
use crate::reference_panel::model::GovernedVcfPrepareReferencePanelSmokeContract;
use crate::reference_panel::model::GOVERNED_VCF_PREPARE_PANEL_ASSET_PROFILE_ID;
use crate::reference_panel::model::GOVERNED_VCF_PREPARE_PANEL_CORPUS_ID;
use crate::reference_panel::model::GOVERNED_VCF_PREPARE_PANEL_INPUT_FIXTURE_ID;
use crate::reference_panel::model::GOVERNED_VCF_PREPARE_PANEL_MAP_ID;
use crate::reference_panel::model::GOVERNED_VCF_PREPARE_PANEL_PANEL_ID;
use crate::reference_panel::model::GOVERNED_VCF_PREPARE_PANEL_STAGE_ID;
use crate::reference_panel::model::GOVERNED_VCF_PREPARE_PANEL_TOOL_ID;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

pub(crate) fn resolve_governed_vcf_prepare_reference_panel_smoke_contract(
    inputs: &HaplotypeBenchmarkInputs,
    requested_tool_id: &str,
) -> Result<GovernedVcfPrepareReferencePanelSmokeContract> {
    let matrix_row = inputs
        .matrix_rows()
        .find(|row| row.stage_id == GOVERNED_VCF_PREPARE_PANEL_STAGE_ID)
        .cloned()
        .ok_or_else(|| {
            anyhow!("VCF stage matrix is missing `{GOVERNED_VCF_PREPARE_PANEL_STAGE_ID}`")
        })?;
    if matrix_row.tool_id != GOVERNED_VCF_PREPARE_PANEL_TOOL_ID {
        bail!(
            "VCF prepare_reference_panel smoke requires retained tool `{GOVERNED_VCF_PREPARE_PANEL_TOOL_ID}`, found `{}` in the governed matrix",
            matrix_row.tool_id
        );
    }
    if requested_tool_id != matrix_row.tool_id {
        bail!(
            "VCF prepare_reference_panel smoke only retains tool `{}` for `{}`; requested `{requested_tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.corpus_id != GOVERNED_VCF_PREPARE_PANEL_CORPUS_ID {
        bail!(
            "VCF prepare_reference_panel smoke requires corpus `{GOVERNED_VCF_PREPARE_PANEL_CORPUS_ID}`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != GOVERNED_VCF_PREPARE_PANEL_ASSET_PROFILE_ID {
        bail!(
            "VCF prepare_reference_panel smoke requires asset profile `{GOVERNED_VCF_PREPARE_PANEL_ASSET_PROFILE_ID}`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec!["prepared_panel".to_string(), "chunks_json".to_string()]
    {
        bail!(
            "VCF prepare_reference_panel smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }
    Ok(GovernedVcfPrepareReferencePanelSmokeContract {
        stage: matrix_row.stage_id,
        tool: matrix_row.tool_id,
        corpus: matrix_row.corpus_id,
        input_fixture: GOVERNED_VCF_PREPARE_PANEL_INPUT_FIXTURE_ID.to_string(),
        panel: GOVERNED_VCF_PREPARE_PANEL_PANEL_ID.to_string(),
        map: GOVERNED_VCF_PREPARE_PANEL_MAP_ID.to_string(),
    })
}
