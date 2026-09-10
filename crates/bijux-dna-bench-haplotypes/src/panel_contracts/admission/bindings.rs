//! panel contracts admission bindings.
use crate::inputs::HaplotypeBenchmarkInputs;
use crate::panel_contracts::model::GovernedVcfPanelWorkflowSmokeContract;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use bijux_dna_bench_corpus::panel_metadata::GOVERNED_VCF_MAP_ID;
use bijux_dna_bench_corpus::panel_metadata::GOVERNED_VCF_PANEL_BUILD_ID;
use bijux_dna_bench_corpus::panel_metadata::GOVERNED_VCF_PANEL_ID;
use bijux_dna_bench_corpus::panel_metadata::GOVERNED_VCF_PANEL_SPECIES_ID;
use bijux_dna_core::ids::StageId;
use bijux_dna_core::ids::ToolId;
use bijux_dna_db_ref::public_api::resolve_map;
use bijux_dna_db_ref::public_api::resolve_panel;
use bijux_dna_db_ref::public_api::validate_imputation_tool_compatibility;

pub(crate) fn resolve_governed_vcf_panel_workflow_smoke_contract(
    inputs: &HaplotypeBenchmarkInputs,
    requested_stage_id: &str,
    requested_tool_id: &str,
    expected_output_id: &str,
) -> Result<GovernedVcfPanelWorkflowSmokeContract> {
    inputs.admit_requested_tool(requested_stage_id, requested_tool_id)?;
    let matrix_row =
        inputs
            .matrix_rows()
            .find(|row| row.stage_id == requested_stage_id)
            .cloned()
            .ok_or_else(|| anyhow!("VCF stage matrix is missing `{requested_stage_id}`"))?;
    let bindings = inputs.bindings();
    if bindings
        .binding(&StageId::new(requested_stage_id), &ToolId::new(requested_tool_id))
        .is_none()
    {
        bail!(
            "VCF panel workflow smoke has no canonical binding for `{requested_stage_id}` / `{requested_tool_id}`"
        );
    }
    let requires_governed_map_compatibility = requested_stage_id == "vcf.phasing"
        || (matches!(requested_stage_id, "vcf.impute" | "vcf.imputation_metrics")
            && requested_tool_id != "beagle");
    if requires_governed_map_compatibility {
        let panel = resolve_panel(
            GOVERNED_VCF_PANEL_SPECIES_ID,
            GOVERNED_VCF_PANEL_BUILD_ID,
            Some(GOVERNED_VCF_PANEL_ID),
        )?;
        let map = resolve_map(
            GOVERNED_VCF_PANEL_SPECIES_ID,
            GOVERNED_VCF_PANEL_BUILD_ID,
            Some(GOVERNED_VCF_MAP_ID),
        )?;
        validate_imputation_tool_compatibility(requested_tool_id, &panel, &map)?;
    }
    if matrix_row.corpus_id != "vcf_production_regression" {
        bail!(
            "VCF panel workflow smoke requires corpus `vcf_production_regression`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != "vcf_cohort_with_panel" {
        bail!(
            "VCF panel workflow smoke requires asset profile `vcf_cohort_with_panel`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec![expected_output_id.to_string()] {
        bail!(
            "VCF panel workflow smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }

    Ok(GovernedVcfPanelWorkflowSmokeContract {
        stage: matrix_row.stage_id,
        tool: requested_tool_id.to_string(),
        corpus: matrix_row.corpus_id,
        panel: GOVERNED_VCF_PANEL_ID.to_string(),
        map: GOVERNED_VCF_MAP_ID.to_string(),
    })
}
