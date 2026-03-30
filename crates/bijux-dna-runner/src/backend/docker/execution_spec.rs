use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1,
};
use bijux_dna_environment::api::{PlatformSpec, ToolImageCatalog};
use serde::Deserialize;

use crate::backend::docker::image_resolution::resolve_image_for_run;
use crate::command_runner::invocation_hash;

#[derive(Debug, Deserialize, Default)]
struct DomainToolYaml {
    #[serde(default)]
    container: DomainToolContainer,
    #[serde(default)]
    command_template: Vec<String>,
    #[serde(default)]
    constraints: Option<ToolConstraints>,
}

#[derive(Debug, Deserialize, Default)]
struct DomainToolContainer {
    #[serde(default)]
    digest: Option<String>,
}

fn load_domain_tool_yaml(tool_id: &str) -> Option<DomainToolYaml> {
    let repo_root = crate::repo_root::resolve_repo_root().ok()?;
    let domain_root = repo_root.join("domain");
    let entries = std::fs::read_dir(domain_root).ok()?;
    let mut matches = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("tools").join(format!("{tool_id}.yaml")))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    matches.sort();
    for path in matches {
        let raw = std::fs::read_to_string(path).ok()?;
        let parsed: DomainToolYaml = bijux_dna_infra::formats::parse_yaml(&raw).ok()?;
        return Some(parsed);
    }
    None
}

fn constraints_are_default(constraints: &ToolConstraints) -> bool {
    constraints.runtime == "local"
        && constraints.mem_gb == 1
        && constraints.tmp_gb == 1
        && constraints.threads == 1
}

/// Build a runtime execution specification from registry and image catalog.
///
/// # Errors
/// Returns an error if stage/tool ids are invalid or registry/catalog lookup fails.
pub fn build_tool_execution_spec(
    stage_id: &str,
    tool_id: &str,
    registry: &bijux_dna_core::contract::ToolRegistry,
    catalog: &impl ToolImageCatalog,
    platform: &PlatformSpec,
) -> Result<ToolExecutionSpecV1> {
    let stage_id =
        bijux_dna_core::ids::StageId::try_from(stage_id).map_err(|err| anyhow!("{err}"))?;
    let tool_id = bijux_dna_core::ids::ToolId::try_from(tool_id).map_err(|err| anyhow!("{err}"))?;
    let manifest = registry
        .tool_by_id(&stage_id, &tool_id)
        .ok_or_else(|| anyhow!("tool {tool_id} missing from manifest for {stage_id}"))?;
    let spec = catalog
        .get(tool_id.as_str())
        .ok_or_else(|| anyhow!("tool {tool_id} missing from images.toml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let mut image_digest = spec.digest.clone();
    let mut command_template = manifest.command_template.clone();
    let mut constraints = manifest.constraints.clone();
    if let Some(domain_tool) = load_domain_tool_yaml(tool_id.as_str()) {
        let DomainToolYaml {
            container,
            command_template: domain_command_template,
            constraints: domain_constraints,
        } = domain_tool;
        if image_digest.is_none() {
            image_digest = container.digest;
        }
        if command_template.is_empty() {
            if !domain_command_template.is_empty() {
                command_template = domain_command_template;
            }
            if constraints_are_default(&constraints) {
                if let Some(domain_constraints) = domain_constraints {
                    constraints = domain_constraints;
                }
            }
        }
    }
    Ok(ToolExecutionSpecV1 {
        tool_id: tool_id.clone(),
        tool_version: spec.version.clone(),
        image: ContainerImageRefV1 {
            image: image.full_name,
            digest: image_digest,
        },
        command: CommandSpecV1 {
            template: command_template,
        },
        resources: constraints,
    })
}

/// Compute a stable invocation hash for a docker execution spec.
///
/// # Errors
/// Returns an error if canonical serialization fails.
pub fn invocation_hash_for_spec(
    spec: &ToolExecutionSpecV1,
    env: &std::collections::BTreeMap<String, String>,
    input_hashes: &[String],
) -> Result<String> {
    let mut identity_env = env.clone();
    identity_env.insert(
        "__BIJUX_CACHE_TOOL_ID".to_string(),
        spec.tool_id.as_str().to_string(),
    );
    identity_env.insert(
        "__BIJUX_CACHE_TOOL_VERSION".to_string(),
        spec.tool_version.clone(),
    );
    let image_digest = spec
        .image
        .digest
        .clone()
        .unwrap_or_else(|| spec.image.image.clone());
    invocation_hash(
        &spec.command.template,
        &identity_env,
        &image_digest,
        input_hashes,
    )
}
