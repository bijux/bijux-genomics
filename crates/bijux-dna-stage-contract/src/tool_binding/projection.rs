use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Context, Result};
use bijux_dna_core::contract::ToolConstraints;
use bijux_dna_core::ids::{StageId, ToolId};
use serde::{Deserialize, Serialize};

use super::{validate_tool_binding, ToolBinding, ToolBindingV1};

pub const EFFECTIVE_TOOL_PARAMETERS_SCHEMA_V1: &str = "bijux.effective_tool_parameters.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectiveToolParameterSourceV1 {
    ScientificDefault,
    InputDerived,
    OperatorOverride,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectiveToolParameterV1 {
    pub value: String,
    pub source: EffectiveToolParameterSourceV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectiveToolParametersV1 {
    pub schema_version: String,
    pub binding_id: String,
    pub stage_id: StageId,
    pub tool_id: ToolId,
    pub allocated_cpus: u32,
    pub resources: ToolConstraints,
    pub parameters: BTreeMap<String, EffectiveToolParameterV1>,
}

impl EffectiveToolParametersV1 {
    #[must_use]
    pub fn render_values(&self) -> BTreeMap<String, String> {
        self.parameters
            .iter()
            .map(|(name, parameter)| (name.clone(), parameter.value.clone()))
            .collect()
    }
}

pub struct ToolParameterProjectionRequest<'a> {
    pub binding: &'a ToolBindingV1,
    pub scientific_defaults: &'a BTreeMap<String, String>,
    pub input_derived: &'a BTreeMap<String, String>,
    pub operator_overrides: &'a BTreeMap<String, String>,
    pub immutable_parameters: &'a BTreeSet<String>,
    pub allocated_cpus: u32,
}

/// Resolve one explainable effective parameter artifact for a tool binding.
///
/// # Errors
/// Returns an error for undeclared parameters, immutable overrides, missing required values, or a
/// thread projection that differs from the scheduler allocation and binding resource contract.
pub fn project_tool_parameters(
    request: &ToolParameterProjectionRequest<'_>,
) -> Result<EffectiveToolParametersV1> {
    validate_tool_binding(request.binding)?;
    if request.allocated_cpus != request.binding.resources().threads {
        bail!("allocated CPUs differ from tool binding threads");
    }
    let declared = request
        .binding
        .parameters()
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect::<BTreeSet<_>>();
    validate_parameter_names(request.scientific_defaults, &declared, "scientific default")?;
    validate_parameter_names(request.input_derived, &declared, "input-derived")?;
    validate_parameter_names(request.operator_overrides, &declared, "operator override")?;

    let mut parameters = BTreeMap::new();
    insert_parameters(
        &mut parameters,
        request.scientific_defaults,
        EffectiveToolParameterSourceV1::ScientificDefault,
    )?;
    insert_parameters(
        &mut parameters,
        request.input_derived,
        EffectiveToolParameterSourceV1::InputDerived,
    )?;
    for (name, value) in request.operator_overrides {
        if request.immutable_parameters.contains(name) {
            bail!("operator override targets immutable parameter `{name}`");
        }
        insert_parameter(
            &mut parameters,
            name,
            value,
            EffectiveToolParameterSourceV1::OperatorOverride,
        )?;
    }
    for parameter in request.binding.parameters().iter().filter(|parameter| parameter.required) {
        if !parameters.contains_key(&parameter.name) {
            bail!("required tool parameter `{}` has no effective value", parameter.name);
        }
    }
    if let Some(threads) = parameters.get("threads") {
        let rendered_threads = threads.value.parse::<u32>().context("parse projected threads")?;
        if rendered_threads != request.allocated_cpus {
            bail!("rendered threads differ from allocated CPUs");
        }
    }
    Ok(EffectiveToolParametersV1 {
        schema_version: EFFECTIVE_TOOL_PARAMETERS_SCHEMA_V1.to_owned(),
        binding_id: request.binding.binding_id().to_owned(),
        stage_id: request.binding.stage_id().clone(),
        tool_id: request.binding.tool_id().clone(),
        allocated_cpus: request.allocated_cpus,
        resources: request.binding.resources().clone(),
        parameters,
    })
}

fn validate_parameter_names(
    values: &BTreeMap<String, String>,
    declared: &BTreeSet<&str>,
    source: &str,
) -> Result<()> {
    for name in values.keys() {
        if !declared.contains(name.as_str()) {
            bail!("{source} uses undeclared tool parameter `{name}`");
        }
    }
    Ok(())
}

fn insert_parameters(
    target: &mut BTreeMap<String, EffectiveToolParameterV1>,
    values: &BTreeMap<String, String>,
    source: EffectiveToolParameterSourceV1,
) -> Result<()> {
    for (name, value) in values {
        insert_parameter(target, name, value, source)?;
    }
    Ok(())
}

fn insert_parameter(
    target: &mut BTreeMap<String, EffectiveToolParameterV1>,
    name: &str,
    value: &str,
    source: EffectiveToolParameterSourceV1,
) -> Result<()> {
    if value.trim().is_empty() {
        bail!("effective tool parameter `{name}` is empty");
    }
    target.insert(name.to_owned(), EffectiveToolParameterV1 { value: value.to_owned(), source });
    Ok(())
}
