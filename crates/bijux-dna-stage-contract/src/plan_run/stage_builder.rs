use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use anyhow::{anyhow, Result};

use bijux_dna_core::contract::{RunSpec, StageSpec, ToolExecutionSpecV1, ToolManifest};
use bijux_dna_core::ids::StageVersion;
use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1};

use crate::{ArtifactRef, PlanDecisionReason, PlanReasonKind, StageIO, StagePlanV1};

/// # Errors
/// Returns an error if the stage has an invalid output contract.
pub fn validate_stage_outputs(stage_spec: &StageSpec, run_spec: &RunSpec) -> Result<()> {
    if stage_spec.stage_id != run_spec.stage {
        return Err(anyhow!(
            "stage {} output contract belongs to {}; expected {}",
            run_spec.stage.0,
            stage_spec.stage_id.0,
            run_spec.stage.0
        ));
    }
    if stage_spec.outputs.is_empty() {
        return Err(anyhow!(
            "stage {} has no declared outputs; planning requires explicit output contract",
            run_spec.stage.0
        ));
    }
    let mut output_names = HashSet::new();
    for output in &stage_spec.outputs {
        if output.name.trim().is_empty() || output.data_type.trim().is_empty() {
            return Err(anyhow!(
                "stage {} has invalid output contract entry (name/data_type must be non-empty)",
                run_spec.stage.0
            ));
        }
        if !output_names.insert(output.name.as_str()) {
            return Err(anyhow!(
                "stage {} has duplicate output contract entry {}",
                run_spec.stage.0,
                output.name
            ));
        }
    }
    Ok(())
}

/// # Errors
/// Returns an error if run parameters cannot be serialized.
pub fn build_stage_plan(
    run_spec: &RunSpec,
    tool_manifest: &ToolManifest,
    stage_spec: &StageSpec,
    run_dir: &Path,
    inputs: Vec<ArtifactRef>,
    outputs: Vec<ArtifactRef>,
) -> Result<StagePlanV1> {
    let params = serde_json::to_value(&run_spec.params).map_err(|err| {
        anyhow!("failed to serialize run parameters for {}: {err}", run_spec.stage.0)
    })?;
    Ok(StagePlanV1 {
        stage_id: run_spec.stage.clone(),
        stage_instance_id: None,
        stage_version: StageVersion(1),
        tool_id: run_spec.tool.clone(),
        tool_version: String::new(),
        image: ContainerImageRefV1 { image: tool_manifest.tool_id.to_string(), digest: None },
        command: CommandSpecV1 { template: tool_manifest.command_template.clone() },
        resources: tool_manifest.constraints.clone(),
        io: StageIO { inputs, outputs },
        out_dir: run_dir.join("stage"),
        params: params.clone(),
        effective_params: params,
        aux_images: BTreeMap::new(),
        reason: PlanDecisionReason {
            kind: PlanReasonKind::Default,
            summary: "planner default".to_string(),
            details: serde_json::json!({
                "runtime_scale": stage_spec.runtime_scale,
                "semantic_kind": stage_spec.semantic_kind,
            }),
        },
    })
}

#[must_use]
pub fn build_tool_execution_spec(
    run_spec: &RunSpec,
    tool_manifest: &ToolManifest,
) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: run_spec.tool.clone(),
        tool_version: String::new(),
        image: ContainerImageRefV1 { image: tool_manifest.tool_id.to_string(), digest: None },
        command: CommandSpecV1 { template: tool_manifest.command_template.clone() },
        resources: tool_manifest.constraints.clone(),
    }
}
