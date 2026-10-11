use std::collections::BTreeSet;

use anyhow::{anyhow, bail, Result};

use super::{OutputDiscoveryV1, ToolArtifactSourceV1, ToolBinding, TOOL_BINDING_SCHEMA_V1};
use crate::canonical_artifact_contract_catalog;

/// Validates that a binding is complete and its template references declared projections.
///
/// # Errors
///
/// Returns an error when required metadata, projections, resources, or placeholders are invalid.
pub fn validate_tool_binding(binding: &impl ToolBinding) -> Result<()> {
    if binding.schema_version() != TOOL_BINDING_SCHEMA_V1 {
        bail!("unsupported tool binding schema `{}`", binding.schema_version());
    }
    require_text(binding.binding_id(), "binding_id")?;
    require_text(binding.stage_id().as_str(), "stage_id")?;
    require_text(binding.tool_id().as_str(), "tool_id")?;
    validate_artifact_source(binding.artifact_source(), binding.tool_id().as_str())?;
    validate_unique_text(binding.capabilities(), "capability", true)?;
    validate_unique_text(binding.limitations(), "limitation", false)?;
    require_text(binding.normalizer_id(), "normalizer_id")?;
    require_text(binding.downstream_probe_id(), "downstream_probe_id")?;
    validate_normalized_output_contract(
        binding.normalized_output_contract_id(),
        binding.stage_id().as_str(),
    )?;
    if binding.resources().threads == 0 || binding.resources().mem_gb == 0 {
        bail!("binding resources require non-zero threads and mem_gb");
    }

    let artifact_catalog = canonical_artifact_contract_catalog();
    artifact_catalog
        .validate()
        .map_err(|error| anyhow!("canonical artifact contract catalog is invalid: {error}"))?;
    let inputs = collect_projection_names(
        binding.inputs().iter().map(|value| (&value.name, &value.artifact_contract_id)),
        "input",
        &artifact_catalog,
    )?;
    let parameters =
        collect_names(binding.parameters().iter().map(|value| &value.name), "parameter")?;
    let outputs = collect_projection_names(
        binding.outputs().iter().map(|value| (&value.name, &value.artifact_contract_id)),
        "output",
        &artifact_catalog,
    )?;
    for output in binding.outputs() {
        if let OutputDiscoveryV1::Glob { pattern } = &output.discovery {
            require_text(pattern, "output glob pattern")?;
        }
    }
    let references = validate_template(binding.command_template(), &inputs, &parameters, &outputs)?;
    require_reference(&references, "executable")?;
    for input in binding.inputs().iter().filter(|value| value.required) {
        let scalar = format!("input:{}", input.name);
        let multiple = format!("inputs:{}", input.name);
        if !references.contains(&scalar) && !references.contains(&multiple) {
            bail!(
                "command_template does not reference required `${{{scalar}}}` or `${{{multiple}}}` projection"
            );
        }
    }
    for parameter in binding.parameters().iter().filter(|value| value.required) {
        require_reference(&references, &format!("parameter:{}", parameter.name))?;
    }
    for output in
        binding.outputs().iter().filter(|value| value.discovery == OutputDiscoveryV1::ExactPath)
    {
        require_reference(&references, &format!("output:{}", output.name))?;
    }
    Ok(())
}

fn validate_normalized_output_contract(contract_id: &str, stage_id: &str) -> Result<()> {
    require_text(contract_id, "normalized_output_contract_id")?;
    let artifact_catalog = canonical_artifact_contract_catalog();
    let benchmark_contract = format!("bijux.benchmark.{stage_id}.bundle.v1");
    if artifact_catalog.contract(contract_id).is_none() && contract_id != benchmark_contract {
        bail!(
            "normalized_output_contract_id is neither a canonical artifact nor the stage benchmark bundle: {contract_id}"
        );
    }
    Ok(())
}

fn validate_artifact_source(source: &ToolArtifactSourceV1, binding_tool_id: &str) -> Result<()> {
    match source {
        ToolArtifactSourceV1::CatalogImage { catalog_tool_id } => {
            require_text(catalog_tool_id.as_str(), "catalog_tool_id")?;
            if catalog_tool_id.as_str() != binding_tool_id {
                bail!(
                    "catalog_tool_id `{catalog_tool_id}` does not match binding tool `{binding_tool_id}`"
                );
            }
            Ok(())
        }
        ToolArtifactSourceV1::ExternalArtifact {
            artifact_kind,
            version_probe,
            checksum_required: _,
        } => {
            require_text(artifact_kind, "external artifact_kind")?;
            if version_probe.is_empty() || version_probe.iter().any(|token| token.trim().is_empty())
            {
                bail!("external artifact version_probe requires non-empty argv");
            }
            if !version_probe.iter().any(|token| token.contains("${executable}")) {
                bail!("external artifact version_probe must reference `${{executable}}`");
            }
            Ok(())
        }
    }
}

fn validate_template(
    template: &[String],
    inputs: &BTreeSet<&str>,
    parameters: &BTreeSet<&str>,
    outputs: &BTreeSet<&str>,
) -> Result<BTreeSet<String>> {
    if template.is_empty() || template.iter().any(String::is_empty) {
        bail!("command_template requires non-empty argv");
    }
    let mut references = BTreeSet::new();
    for token in template {
        for placeholder in placeholders(token)? {
            references.insert(placeholder.to_string());
            if placeholder == "executable" {
                continue;
            }
            let (kind, name) = placeholder
                .split_once(':')
                .ok_or_else(|| anyhow!("invalid binding placeholder `${{{placeholder}}}`"))?;
            let declared = match kind {
                "input" | "inputs" => inputs.contains(name),
                "parameter" => parameters.contains(name),
                "output" => outputs.contains(name),
                _ => bail!("unknown binding placeholder kind `{kind}`"),
            };
            if !declared {
                bail!("command_template references undeclared {kind} `{name}`");
            }
            let complete_token = format!("${{{placeholder}}}");
            if kind == "inputs" && token != &complete_token {
                bail!("multi-path input projection must occupy one complete command token");
            }
        }
    }
    Ok(references)
}

fn require_reference(references: &BTreeSet<String>, placeholder: &str) -> Result<()> {
    if !references.contains(placeholder) {
        bail!("command_template does not reference required `${{{placeholder}}}` projection");
    }
    Ok(())
}

fn placeholders(token: &str) -> Result<Vec<&str>> {
    let mut values = Vec::new();
    let mut remaining = token;
    while let Some(start) = remaining.find("${") {
        let placeholder = &remaining[start + 2..];
        let end = placeholder
            .find('}')
            .ok_or_else(|| anyhow!("unterminated placeholder in `{token}`"))?;
        values.push(&placeholder[..end]);
        remaining = &placeholder[end + 1..];
    }
    Ok(values)
}

fn collect_projection_names<'a>(
    values: impl Iterator<Item = (&'a String, &'a String)>,
    kind: &str,
    artifact_catalog: &crate::ArtifactContractCatalogV1,
) -> Result<BTreeSet<&'a str>> {
    let mut names = BTreeSet::new();
    for (name, contract_id) in values {
        require_text(name, &format!("{kind} name"))?;
        require_text(contract_id, &format!("{kind} artifact_contract_id"))?;
        if artifact_catalog.contract(contract_id).is_none() {
            bail!("{kind} `{name}` references unknown artifact contract `{contract_id}`");
        }
        if !names.insert(name.as_str()) {
            bail!("duplicate {kind} projection `{name}`");
        }
    }
    Ok(names)
}

fn collect_names<'a>(
    values: impl Iterator<Item = &'a String>,
    kind: &str,
) -> Result<BTreeSet<&'a str>> {
    let mut names = BTreeSet::new();
    for name in values {
        require_text(name, &format!("{kind} name"))?;
        if !names.insert(name.as_str()) {
            bail!("duplicate {kind} projection `{name}`");
        }
    }
    Ok(names)
}

fn validate_unique_text(values: &[String], kind: &str, require_one: bool) -> Result<()> {
    if require_one && values.is_empty() {
        bail!("binding requires at least one {kind}");
    }
    let mut seen = BTreeSet::new();
    for value in values {
        require_text(value, kind)?;
        if !seen.insert(value) {
            bail!("duplicate {kind} `{value}`");
        }
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{field} must not be empty");
    }
    Ok(())
}
