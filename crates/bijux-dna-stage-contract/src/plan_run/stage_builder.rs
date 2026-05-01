use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use anyhow::{anyhow, Result};

use bijux_dna_core::contract::{RunSpec, StageSpec, ToolExecutionSpecV1, ToolManifest};
use bijux_dna_core::ids::StageVersion;
use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1};

use crate::{
    ArtifactRef, PlanDecisionReason, PlanReasonKind, StageArtifactPromiseV1, StageIO, StagePlanV1,
    StageProvenanceV1,
};

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
    validate_ports(&stage_spec.inputs, run_spec.stage.as_str(), "input")?;
    let mut output_names = HashSet::new();
    for output in &stage_spec.outputs {
        validate_port(output, run_spec.stage.as_str(), "output")?;
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
/// Returns an error if a canonical stage contract is internally inconsistent.
pub fn validate_stage_contract(stage_spec: &StageSpec, tool_manifest: &ToolManifest) -> Result<()> {
    validate_ports(&stage_spec.inputs, stage_spec.stage_id.as_str(), "input")?;
    validate_ports(&stage_spec.outputs, stage_spec.stage_id.as_str(), "output")?;
    for report in &stage_spec.report_contracts {
        if report.report_id.trim().is_empty() || report.schema_version.trim().is_empty() {
            return Err(anyhow!(
                "stage {} has malformed report contract; report_id and schema_version are required",
                stage_spec.stage_id.0
            ));
        }
    }
    if !tool_manifest.supported_modes.is_empty()
        && !tool_manifest.supported_modes.contains(&stage_spec.operating_mode)
    {
        return Err(anyhow!(
            "stage {} requested unsupported operating mode {:?} for backend {}",
            stage_spec.stage_id.0,
            stage_spec.operating_mode,
            tool_manifest.tool_id.0
        ));
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
    validate_stage_contract(stage_spec, tool_manifest)?;
    let params = serde_json::to_value(&run_spec.params).map_err(|err| {
        anyhow!("failed to serialize run parameters for {}: {err}", run_spec.stage.0)
    })?;
    let canonical_params = stage_spec.canonicalize_parameters(&run_spec.params)?;
    let canonical_contract = stage_spec.canonical_contract(tool_manifest);
    let input_artifact_ids =
        inputs.iter().map(|artifact| artifact.name.to_string()).collect::<Vec<_>>();
    let output_promises = outputs
        .iter()
        .map(|artifact| StageArtifactPromiseV1 {
            artifact_id: artifact.name.to_string(),
            role: artifact.role,
            path: artifact.path.to_string_lossy().to_string(),
            optional: artifact.optional,
        })
        .collect::<Vec<_>>();
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
        effective_params: canonical_params.normalized_json.clone(),
        operating_mode: stage_spec.operating_mode,
        aux_images: BTreeMap::new(),
        canonical_contract: Some(canonical_contract.clone()),
        provenance: Some(StageProvenanceV1 {
            stage_id: stage_spec.stage_id.clone(),
            stage_family: stage_spec.stage_family,
            semantic_kind: stage_spec.semantic_kind,
            backend_tool_id: tool_manifest.tool_id.clone(),
            backend_version_policy: tool_manifest.backend_version_policy,
            operating_mode: stage_spec.operating_mode,
            tool_surface: tool_manifest.tool_id.to_string(),
            effective_parameters_json: canonical_params.normalized_json.clone(),
            effective_parameters_hash: canonical_params.hash.clone(),
            input_artifact_ids,
            output_promises,
            report_contracts: stage_spec.report_contracts.clone(),
        }),
        reason: PlanDecisionReason {
            kind: PlanReasonKind::Default,
            summary: "planner default".to_string(),
            details: serde_json::json!({
                "stage_family": stage_spec.stage_family,
                "runtime_scale": stage_spec.runtime_scale,
                "semantic_kind": stage_spec.semantic_kind,
                "operating_mode": stage_spec.operating_mode,
                "canonical_parameters_hash": canonical_params.hash,
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

fn validate_ports(
    ports: &[bijux_dna_core::contract::PortSpec],
    stage_id: &str,
    surface: &str,
) -> Result<()> {
    for port in ports {
        validate_port(port, stage_id, surface)?;
    }
    Ok(())
}

fn validate_port(
    port: &bijux_dna_core::contract::PortSpec,
    stage_id: &str,
    surface: &str,
) -> Result<()> {
    if port.name.trim().is_empty() || port.data_type.trim().is_empty() {
        return Err(anyhow!(
            "stage {stage_id} has invalid {surface} contract entry (name/data_type must be non-empty)"
        ));
    }
    if !port.artifact_role.is_typed() {
        return Err(anyhow!(
            "stage {stage_id} has untyped {surface} contract entry {}; set artifact_role explicitly",
            port.name
        ));
    }
    Ok(())
}
