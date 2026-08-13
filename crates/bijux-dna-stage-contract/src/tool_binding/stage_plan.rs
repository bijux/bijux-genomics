use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use bijux_dna_core::contract::{ArtifactRef, StageIO};
use bijux_dna_core::ids::{StageVersion, StepId};
use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1};

use crate::{PlanDecisionReason, PlanReasonKind, StagePlanV1};

use super::{
    render_tool_command, validate_effective_tool_parameters, validate_tool_binding,
    EffectiveToolParametersV1, ToolBinding, ToolBindingRenderContext, ToolBindingV1,
};

struct BindingInputPaths {
    scalar: BTreeMap<String, String>,
    multiple: BTreeMap<String, Vec<String>>,
}

/// Complete, execution-owner-supplied authority for one binding-backed stage plan.
pub struct BindingStagePlanRequest<'a> {
    pub binding: &'a ToolBindingV1,
    pub effective_parameters: &'a EffectiveToolParametersV1,
    pub image: ContainerImageRefV1,
    pub executable: String,
    pub tool_version: String,
    pub stage_instance_id: Option<StepId>,
    pub inputs: Vec<ArtifactRef>,
    pub outputs: Vec<ArtifactRef>,
    pub out_dir: PathBuf,
}

/// Construct one immutable stage plan entirely from admitted binding and execution authority.
///
/// The constructor is deliberately generic: adding a tool requires a new binding, not a branch in
/// the lifecycle engine. Inputs and outputs must use the exact binding names and artifact contracts,
/// projected parameters must belong to the same stage/tool/resources, and rendered argv is frozen in
/// the returned plan.
///
/// # Errors
/// Returns an error when binding, parameters, artifacts, image, executable, or output-root authority
/// is missing or inconsistent.
pub fn build_binding_stage_plan(request: BindingStagePlanRequest<'_>) -> Result<StagePlanV1> {
    validate_tool_binding(request.binding).context("validate stage plan tool binding")?;
    validate_effective_tool_parameters(request.effective_parameters)
        .context("validate stage plan effective parameters")?;
    validate_authority_identity(&request)?;

    let mut inputs = request.inputs;
    inputs.sort_by(|left, right| {
        left.name.as_str().cmp(right.name.as_str()).then_with(|| left.path.cmp(&right.path))
    });
    let mut outputs = request.outputs;
    outputs.sort_by(|left, right| {
        left.name.as_str().cmp(right.name.as_str()).then_with(|| left.path.cmp(&right.path))
    });
    let input_paths = validate_inputs(request.binding, &inputs)?;
    let output_paths = validate_outputs(request.binding, &outputs)?;
    let rendered_parameters = request.effective_parameters.render_values();
    let command = render_tool_command(
        request.binding,
        &ToolBindingRenderContext {
            executable: request.executable,
            inputs: input_paths.scalar,
            input_sets: input_paths.multiple,
            parameters: rendered_parameters.clone(),
            outputs: output_paths,
        },
    )
    .context("render binding-backed stage command")?;
    let params = serde_json::to_value(&rendered_parameters)
        .context("serialize binding-backed stage parameters")?;
    let effective_params = serde_json::to_value(request.effective_parameters)
        .context("serialize effective stage parameter authority")?;

    Ok(StagePlanV1 {
        stage_id: request.binding.stage_id().clone(),
        stage_instance_id: request.stage_instance_id,
        stage_version: StageVersion(1),
        tool_id: request.binding.tool_id().clone(),
        tool_version: request.tool_version,
        image: request.image,
        command: CommandSpecV1 { template: command },
        resources: request.binding.resources().clone(),
        io: StageIO { inputs, outputs },
        out_dir: request.out_dir,
        params,
        effective_params,
        operating_mode: Default::default(),
        aux_images: BTreeMap::new(),
        canonical_contract: None,
        provenance: None,
        reason: PlanDecisionReason {
            kind: PlanReasonKind::Compatibility,
            summary: "canonical tool binding selected for immutable execution".to_string(),
            details: serde_json::json!({
                "binding_id": request.binding.binding_id(),
                "normalizer_id": request.binding.normalizer_id(),
                "downstream_probe_id": request.binding.downstream_probe_id(),
            }),
        },
    })
}

fn validate_authority_identity(request: &BindingStagePlanRequest<'_>) -> Result<()> {
    let effective = request.effective_parameters;
    let binding = request.binding;
    if effective.binding_id != binding.binding_id()
        || effective.stage_id != *binding.stage_id()
        || effective.tool_id != *binding.tool_id()
        || effective.resources != *binding.resources()
    {
        bail!("effective parameter authority differs from the tool binding");
    }
    if request.image.image.trim().is_empty()
        || request.image.digest.as_deref().is_some_and(|digest| digest.trim().is_empty())
    {
        bail!("binding-backed stage image identity is empty");
    }
    if request.executable.trim().is_empty() || request.tool_version.trim().is_empty() {
        bail!("binding-backed stage executable or tool version is empty");
    }
    if request.out_dir.as_os_str().is_empty() {
        bail!("binding-backed stage output root is empty");
    }
    Ok(())
}

fn validate_inputs(
    binding: &impl ToolBinding,
    inputs: &[ArtifactRef],
) -> Result<BindingInputPaths> {
    let declared = binding
        .inputs()
        .iter()
        .map(|input| (input.name.as_str(), input))
        .collect::<BTreeMap<_, _>>();
    let mut observed = BTreeMap::<String, Vec<String>>::new();
    for input in inputs {
        let name = input.name.as_str();
        let authority = declared
            .get(name)
            .with_context(|| format!("stage plan input `{name}` is not declared by the binding"))?;
        if input.path.as_os_str().is_empty()
            || input.artifact_contract_id.as_deref()
                != Some(authority.artifact_contract_id.as_str())
        {
            bail!("stage plan input `{name}` differs from binding artifact authority");
        }
        observed.entry(name.to_string()).or_default().push(input.path.display().to_string());
    }
    let missing = binding
        .inputs()
        .iter()
        .filter(|input| input.required && !observed.contains_key(&input.name))
        .map(|input| input.name.as_str())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        bail!("stage plan omits required binding inputs: {}", missing.join(", "));
    }
    let scalar = observed
        .iter()
        .filter(|(_, paths)| paths.len() == 1)
        .map(|(name, paths)| (name.clone(), paths[0].clone()))
        .collect();
    Ok(BindingInputPaths { scalar, multiple: observed })
}

fn validate_outputs(
    binding: &impl ToolBinding,
    outputs: &[ArtifactRef],
) -> Result<BTreeMap<String, String>> {
    let declared = binding
        .outputs()
        .iter()
        .map(|output| (output.name.as_str(), output.artifact_contract_id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut observed_names = BTreeSet::new();
    let mut observed = BTreeMap::new();
    for output in outputs {
        let name = output.name.as_str();
        let contract = declared.get(name).with_context(|| {
            format!("stage plan output `{name}` is not declared by the binding")
        })?;
        if output.path.as_os_str().is_empty()
            || output.artifact_contract_id.as_deref() != Some(*contract)
        {
            bail!("stage plan output `{name}` differs from binding artifact authority");
        }
        if !observed_names.insert(name) {
            bail!("stage plan output `{name}` is duplicated");
        }
        observed.insert(name.to_string(), output.path.display().to_string());
    }
    if observed.len() != declared.len() || declared.keys().any(|name| !observed.contains_key(*name))
    {
        bail!("stage plan outputs do not close the binding output contract");
    }
    Ok(observed)
}
