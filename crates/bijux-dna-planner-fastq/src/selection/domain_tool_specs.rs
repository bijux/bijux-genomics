use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, StageId, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DomainToolYaml {
    tool_id: String,
    default_version: String,
    container: DomainToolContainer,
    #[serde(default)]
    command_template: Vec<String>,
    #[serde(default)]
    constraints: Option<ToolConstraints>,
    #[serde(default)]
    stage_id: Option<String>,
    #[serde(default)]
    stage_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DomainToolContainer {
    image: String,
    #[serde(default)]
    digest: Option<String>,
}

/// # Errors
/// Returns an error if the governed FASTQ domain tool YAML cannot be read, does not match the
/// requested stage/tool pair, or omits required execution-spec fields.
pub(crate) fn load_fastq_domain_tool_execution_spec(
    repo_root: &Path,
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Result<ToolExecutionSpecV1> {
    let yaml_path =
        repo_root.join("domain").join("fastq").join("tools").join(format!("{tool_id}.yaml"));
    let raw = std::fs::read_to_string(&yaml_path)
        .with_context(|| format!("read governed tool yaml {}", yaml_path.display()))?;
    let parsed: DomainToolYaml = bijux_dna_infra::formats::parse_yaml(&raw)
        .with_context(|| format!("parse governed tool yaml {}", yaml_path.display()))?;

    if parsed.tool_id != tool_id.as_str() {
        return Err(anyhow!(
            "governed tool yaml {} declares tool_id {}, expected {}",
            yaml_path.display(),
            parsed.tool_id,
            tool_id.as_str()
        ));
    }

    let mut admitted_stage_ids = parsed.stage_ids;
    if let Some(single_stage_id) = parsed.stage_id {
        admitted_stage_ids.push(single_stage_id);
    }
    if !admitted_stage_ids.iter().any(|candidate| candidate == stage_id.as_str()) {
        return Err(anyhow!(
            "governed tool yaml {} does not admit stage {}",
            yaml_path.display(),
            stage_id.as_str()
        ));
    }

    Ok(ToolExecutionSpecV1 {
        tool_id: tool_id.clone(),
        tool_version: parsed.default_version,
        image: ContainerImageRefV1 {
            image: parsed.container.image,
            digest: parsed.container.digest,
        },
        command: CommandSpecV1 { template: parsed.command_template },
        resources: parsed.constraints.unwrap_or_default(),
    })
}
