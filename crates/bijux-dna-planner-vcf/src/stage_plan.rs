use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::ids::{StageId, StageVersion, ToolId};
use bijux_dna_core::prelude::{StageIO, StageOperatingMode, ToolConstraints};
use bijux_dna_db_ref::{MapCatalogEntry, ReferenceBundle};
use bijux_dna_domain_vcf::taxonomy::{CoverageRegime, VcfDomainStage};
use bijux_dna_stage_contract::{PlanDecisionReason, PlanReasonKind, StagePlanV1};

use crate::api::{ChunkPlanSettings, VcfPanelLock};
use crate::chunk_plan::RegionChunkPlan;
use crate::params::{
    apply_stage_param_overrides, attach_reference_provenance, stage_params,
    validate_generated_stage_params,
};
use crate::stage_io::{stage_command, stage_inputs_for, stage_outputs_for};
use crate::tool_catalog::image_for_tool;

fn stage_resources(stage: VcfDomainStage) -> ToolConstraints {
    ToolConstraints {
        runtime: "docker".to_string(),
        mem_gb: if matches!(stage, VcfDomainStage::Impute | VcfDomainStage::Imputation) {
            16
        } else {
            4
        },
        tmp_gb: 8,
        threads: if matches!(
            stage,
            VcfDomainStage::Impute | VcfDomainStage::Imputation | VcfDomainStage::Phasing
        ) {
            8
        } else {
            2
        },
    }
}

/// # Errors
/// Returns an error if the stage plan cannot be assembled for the selected tool and inputs.
#[allow(clippy::too_many_arguments)]
pub fn build_stage_plan(
    stage: VcfDomainStage,
    input_vcf: &Path,
    out_dir: &Path,
    tool: &str,
    call_bam: Option<&Path>,
    call_bam_index: Option<&Path>,
    reference_fasta: Option<&Path>,
    reference_panel_vcf: Option<&Path>,
    coverage: CoverageRegime,
    selected_panel: Option<&VcfPanelLock>,
    map: &MapCatalogEntry,
    bundle: &ReferenceBundle,
    species_id: &str,
    build_id: &str,
    stage_param_overrides: &std::collections::BTreeMap<String, serde_json::Value>,
    chunking: &ChunkPlanSettings,
    chunks: &[RegionChunkPlan],
    selection_rule: &str,
) -> Result<StagePlanV1> {
    let stage_id = stage.as_str().to_string();
    let io_inputs = stage_inputs_for(
        stage,
        input_vcf,
        out_dir,
        call_bam,
        call_bam_index,
        reference_fasta,
        reference_panel_vcf,
    )?;
    let io_outputs = stage_outputs_for(stage, out_dir);
    let params = attach_reference_provenance(
        apply_stage_param_overrides(
            &stage_id,
            stage_params(stage, tool, coverage, selected_panel, map, chunking, chunks),
            stage_param_overrides.get(&stage_id),
        )
        .map_err(|err| anyhow!("stage param override validation failed for {stage_id}: {err}"))?,
        species_id,
        build_id,
        bundle,
    );
    validate_generated_stage_params(&stage_id, &params)?;
    Ok(StagePlanV1 {
        stage_id: StageId::new(stage_id),
        stage_instance_id: None,
        stage_version: StageVersion(2),
        tool_id: ToolId::new(tool.to_string()),
        tool_version: "1.0".to_string(),
        image: image_for_tool(tool),
        command: stage_command(stage, tool, &io_inputs, &io_outputs)?,
        resources: stage_resources(stage),
        io: StageIO { inputs: io_inputs, outputs: io_outputs },
        out_dir: out_dir.join(stage.as_str().replace('.', "_")),
        params: params.clone(),
        effective_params: params,
        operating_mode: StageOperatingMode::Enforced,
        aux_images: std::collections::BTreeMap::new(),
        canonical_contract: None,
        provenance: None,
        reason: PlanDecisionReason {
            kind: PlanReasonKind::InputAssessed,
            summary: format!(
                "coverage regime {:?} selected tool {} for {} ({})",
                coverage,
                tool,
                stage.as_str(),
                selection_rule
            ),
            details: serde_json::json!({
                "coverage_regime": coverage,
                "stage_kind": stage.taxonomy().kind,
                "selection_rule": selection_rule,
            }),
        },
    })
}
