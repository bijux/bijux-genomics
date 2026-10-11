//! Pure binding input/output admission and command parameter projection.
use crate::qualification::specification::QualificationSpecificationSourceV1;
use crate::qualification::specification::{
    admission::{require_path_under, require_safe_relative, require_sha256},
    QualificationSpecificationInputV1,
};
use anyhow::{bail, Context, Result};
use bijux_dna_stage_contract::{ToolBinding, ToolBindingRenderContext};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) fn render_context(
    binding: &impl ToolBinding,
    source: &QualificationSpecificationSourceV1,
    executable: &str,
) -> Result<ToolBindingRenderContext> {
    let declared_inputs =
        binding.inputs().iter().map(|input| input.name.as_str()).collect::<BTreeSet<_>>();
    let mut inputs = BTreeMap::new();
    for input in &source.inputs {
        if !declared_inputs.contains(input.binding_input.as_str()) {
            bail!(
                "qualification source supplies undeclared binding input `{}`",
                input.binding_input
            );
        }
        let binding_input = binding
            .inputs()
            .iter()
            .find(|value| value.name == input.binding_input)
            .context("declared qualification binding input disappeared")?;
        if binding_input.artifact_contract_id != input.artifact_contract_id {
            bail!("qualification input `{}` has the wrong artifact contract", input.binding_input);
        }
        if inputs
            .insert(input.binding_input.clone(), input.mount_path.display().to_string())
            .is_some()
        {
            bail!("qualification source duplicates binding input `{}`", input.binding_input);
        }
        validate_input_declaration(input, &source.governed_study_root)?;
    }
    for required in binding.inputs().iter().filter(|input| input.required) {
        if !inputs.contains_key(&required.name) {
            bail!("qualification source omits required binding input `{}`", required.name);
        }
    }
    let declared_parameters =
        binding.parameters().iter().map(|value| value.name.as_str()).collect::<BTreeSet<_>>();
    if source.parameters.keys().any(|key| !declared_parameters.contains(key.as_str())) {
        bail!("qualification source supplies an undeclared binding parameter");
    }
    for required in binding.parameters().iter().filter(|value| value.required) {
        if source.parameters.get(&required.name).is_none_or(|value| value.trim().is_empty()) {
            bail!("qualification source omits required binding parameter `{}`", required.name);
        }
    }
    let declared_outputs =
        binding.outputs().iter().map(|output| output.name.as_str()).collect::<BTreeSet<_>>();
    let mut outputs = BTreeMap::new();
    for output in &source.outputs {
        if !declared_outputs.contains(output.binding_output.as_str()) {
            bail!(
                "qualification source supplies undeclared binding output `{}`",
                output.binding_output
            );
        }
        let binding_output = binding
            .outputs()
            .iter()
            .find(|value| value.name == output.binding_output)
            .context("declared qualification binding output disappeared")?;
        if binding_output.artifact_contract_id != output.artifact_contract_id {
            bail!(
                "qualification output `{}` has the wrong artifact contract",
                output.binding_output
            );
        }
        require_safe_relative(&output.path, "qualification output")?;
        if outputs
            .insert(output.binding_output.clone(), format!("/result/{}", output.path.display()))
            .is_some()
        {
            bail!("qualification source duplicates binding output `{}`", output.binding_output);
        }
    }
    if outputs.len() != binding.outputs().len() {
        bail!("qualification source must project every binding output exactly once");
    }
    Ok(ToolBindingRenderContext {
        executable: executable.to_string(),
        inputs,
        input_sets: BTreeMap::new(),
        parameters: source.parameters.clone(),
        outputs,
    })
}

pub(super) fn validate_negative_command(
    source: &QualificationSpecificationSourceV1,
    expected_executable: &str,
) -> Result<()> {
    if source.negative_command.first().map(String::as_str) != Some(expected_executable)
        || source.negative_command.iter().any(String::is_empty)
    {
        bail!("qualification negative command must invoke the exact catalog executable");
    }
    Ok(())
}

pub(super) fn validate_input_declaration(
    input: &QualificationSpecificationInputV1,
    study_root: &Path,
) -> Result<()> {
    if input.id.trim().is_empty() || !input.path.is_absolute() {
        bail!("qualification input identity and source path must be complete");
    }
    require_path_under(&input.path, study_root, "qualification input")?;
    if input.mount_path.parent() != Some(Path::new("/input")) {
        bail!("qualification input mount must be one file under /input");
    }
    require_sha256(&input.sha256, "qualification input")?;
    Ok(())
}
