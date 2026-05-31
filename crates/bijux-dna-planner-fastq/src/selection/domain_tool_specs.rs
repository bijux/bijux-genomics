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
    #[serde(default)]
    container: Option<DomainToolContainer>,
    #[serde(default)]
    command_template: Vec<String>,
    #[serde(default)]
    constraints: Option<ToolConstraints>,
    #[serde(default)]
    install_kind: Option<String>,
    #[serde(default)]
    help_cmd: Option<String>,
    #[serde(default)]
    version_cmd: Option<String>,
    #[serde(default)]
    stage_id: Option<String>,
    #[serde(default)]
    stage_ids: Vec<String>,
    #[serde(default)]
    planned_stage_ids: Vec<String>,
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

    let mut admitted_stage_ids = parsed.stage_ids.clone();
    if let Some(single_stage_id) = parsed.stage_id.as_ref() {
        admitted_stage_ids.push(single_stage_id.clone());
    }
    admitted_stage_ids.extend(parsed.planned_stage_ids.iter().cloned());
    if !admitted_stage_ids.iter().any(|candidate| candidate == stage_id.as_str()) {
        return Err(anyhow!(
            "governed tool yaml {} does not admit stage {}",
            yaml_path.display(),
            stage_id.as_str()
        ));
    }

    let workspace_binary_entrypoint = workspace_binary_entrypoint(&parsed)?;
    let command_template = if parsed.command_template.is_empty() {
        vec![workspace_binary_entrypoint.clone()]
    } else {
        parsed.command_template.clone()
    };
    let image = match parsed.container {
        Some(container) => ContainerImageRefV1 {
            image: container.image,
            digest: container.digest,
        },
        None => ContainerImageRefV1 {
            image: workspace_binary_entrypoint,
            digest: None,
        },
    };

    Ok(ToolExecutionSpecV1 {
        tool_id: tool_id.clone(),
        tool_version: parsed.default_version,
        image,
        command: CommandSpecV1 { template: command_template },
        resources: parsed.constraints.unwrap_or_default(),
    })
}

fn workspace_binary_entrypoint(parsed: &DomainToolYaml) -> Result<String> {
    let install_kind = parsed.install_kind.as_deref().unwrap_or("container");
    if install_kind != "workspace_binary" {
        return Err(anyhow!(
            "governed tool yaml for {} omits required container metadata",
            parsed.tool_id
        ));
    }

    parsed
        .help_cmd
        .as_deref()
        .or(parsed.version_cmd.as_deref())
        .and_then(|command| command.split_whitespace().next())
        .map(str::to_string)
        .ok_or_else(|| {
            anyhow!(
                "workspace-binary tool yaml for {} must declare help_cmd or version_cmd",
                parsed.tool_id
            )
        })
}

#[cfg(test)]
mod tests {
    use super::load_fastq_domain_tool_execution_spec;
    use anyhow::Result;
    use bijux_dna_core::prelude::{StageId, ToolId};
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap_or_else(|| panic!("workspace root"))
            .to_path_buf()
    }

    #[test]
    fn load_fastq_domain_tool_execution_spec_accepts_planned_workspace_binary_stage() -> Result<()> {
        let repo_root = repo_root();
        let stage_id = StageId::new("fastq.detect_duplicates_premerge".to_string());
        let tool_id = ToolId::new("bijux_dna");

        let spec = load_fastq_domain_tool_execution_spec(&repo_root, &stage_id, &tool_id)?;

        assert_eq!(spec.tool_id.as_str(), "bijux_dna");
        assert_eq!(spec.command.template, vec!["bijux-dna".to_string()]);
        assert_eq!(spec.image.image, "bijux-dna");
        assert!(spec.image.digest.is_none());
        Ok(())
    }
}
