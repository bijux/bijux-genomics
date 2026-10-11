use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, StageId, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BamDomainToolSupportLevel {
    Supported,
    Planned,
}

impl BamDomainToolSupportLevel {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Planned => "planned",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BamDomainToolContractMetadata {
    pub tool_id: ToolId,
    pub support_level: BamDomainToolSupportLevel,
    pub stage_ids: Vec<StageId>,
    pub planned_stage_ids: Vec<StageId>,
}

impl BamDomainToolContractMetadata {
    #[must_use]
    pub fn pair_support_level(&self, stage_id: &StageId) -> BamDomainToolSupportLevel {
        if self.planned_stage_ids.iter().any(|candidate| candidate == stage_id)
            || self.support_level == BamDomainToolSupportLevel::Planned
        {
            BamDomainToolSupportLevel::Planned
        } else {
            BamDomainToolSupportLevel::Supported
        }
    }
}

#[derive(Debug, Deserialize)]
struct DomainToolYaml {
    tool_id: String,
    default_version: String,
    status: String,
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
/// Returns an error if the governed BAM domain tool YAML cannot be read or omits required
/// support metadata.
pub fn load_bam_domain_tool_contract_metadata(
    repo_root: &Path,
    tool_id: &ToolId,
) -> Result<BamDomainToolContractMetadata> {
    load_bam_domain_tool_definition(repo_root, tool_id)?.contract_metadata()
}

/// # Errors
/// Returns an error if the governed BAM domain tool YAML cannot be read, does not match the
/// requested stage/tool pair, or omits required execution-spec fields.
pub fn load_bam_domain_tool_execution_spec(
    repo_root: &Path,
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Result<ToolExecutionSpecV1> {
    load_bam_domain_tool_spec_inner(repo_root, stage_id, tool_id, false)
}

/// # Errors
/// Returns an error if the governed BAM domain tool YAML cannot be read or does not match the
/// requested stage/tool pair. Unlike the execution-spec loader, this planning-only variant
/// tolerates tool records that omit container metadata.
pub fn load_bam_domain_tool_planning_spec(
    repo_root: &Path,
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Result<ToolExecutionSpecV1> {
    load_bam_domain_tool_spec_inner(repo_root, stage_id, tool_id, true)
}

fn load_bam_domain_tool_spec_inner(
    repo_root: &Path,
    stage_id: &StageId,
    tool_id: &ToolId,
    allow_placeholder_image: bool,
) -> Result<ToolExecutionSpecV1> {
    load_bam_domain_tool_definition(repo_root, tool_id)?
        .project_spec(stage_id, allow_placeholder_image)
}

fn default_command_entrypoint(
    parsed: &DomainToolYaml,
    allow_placeholder_image: bool,
) -> Result<String> {
    let install_kind = parsed.install_kind.as_deref().unwrap_or("container");
    if install_kind == "workspace_binary" {
        return workspace_binary_entrypoint(parsed);
    }
    if parsed.container.is_none() && !allow_placeholder_image {
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
                "container tool yaml for {} must declare help_cmd or version_cmd when command_template is omitted",
                parsed.tool_id
            )
        })
}

fn workspace_binary_entrypoint(parsed: &DomainToolYaml) -> Result<String> {
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

fn load_domain_tool_yaml(repo_root: &Path, tool_id: &ToolId) -> Result<DomainToolYaml> {
    let yaml_path =
        repo_root.join("domain").join("bam").join("tools").join(format!("{tool_id}.yaml"));
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

    Ok(parsed)
}

/// One identity-checked tool YAML observation shared by metadata and spec projections.
#[derive(Debug)]
pub struct BamDomainToolDefinition {
    tool_id: ToolId,
    source_path: PathBuf,
    parsed: DomainToolYaml,
}

/// Read and parse the current governed BAM tool definition.
///
/// # Errors
/// Rejects unreadable, malformed, or differently identified tool YAML.
pub fn load_bam_domain_tool_definition(
    repo_root: &Path,
    tool_id: &ToolId,
) -> Result<BamDomainToolDefinition> {
    let parsed = load_domain_tool_yaml(repo_root, tool_id)?;
    Ok(BamDomainToolDefinition {
        tool_id: tool_id.clone(),
        source_path: repo_root
            .join("domain")
            .join("bam")
            .join("tools")
            .join(format!("{tool_id}.yaml")),
        parsed,
    })
}

impl BamDomainToolDefinition {
    /// Project support metadata from this observation.
    ///
    /// # Errors
    /// Rejects unrecognized support status values.
    pub fn contract_metadata(&self) -> Result<BamDomainToolContractMetadata> {
        let parsed = &self.parsed;
        let tool_id = &self.tool_id;
        let support_level = match parsed.status.as_str() {
            "supported" => BamDomainToolSupportLevel::Supported,
            "planned" => BamDomainToolSupportLevel::Planned,
            other => {
                return Err(anyhow!(
                    "governed BAM tool yaml {} declares unsupported status `{other}`",
                    tool_id.as_str()
                ))
            }
        };

        let stage_ids = parsed.stage_ids.iter().cloned().map(StageId::new).collect::<Vec<_>>();
        let planned_stage_ids =
            parsed.planned_stage_ids.iter().cloned().map(StageId::new).collect::<Vec<_>>();

        Ok(BamDomainToolContractMetadata {
            tool_id: tool_id.clone(),
            support_level,
            stage_ids,
            planned_stage_ids,
        })
    }

    /// Project an executable stage binding from this observation.
    ///
    /// # Errors
    /// Rejects an unadmitted stage or missing execution fields.
    pub fn execution_spec(&self, stage_id: &StageId) -> Result<ToolExecutionSpecV1> {
        self.project_spec(stage_id, false)
    }

    /// Project a planning binding, permitting the existing placeholder-image policy.
    ///
    /// # Errors
    /// Rejects an unadmitted stage or missing command fields.
    pub fn planning_spec(&self, stage_id: &StageId) -> Result<ToolExecutionSpecV1> {
        self.project_spec(stage_id, true)
    }

    fn project_spec(
        &self,
        stage_id: &StageId,
        allow_placeholder_image: bool,
    ) -> Result<ToolExecutionSpecV1> {
        let parsed = &self.parsed;
        let tool_id = &self.tool_id;
        let yaml_path = &self.source_path;

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

        let default_entrypoint = if parsed.command_template.is_empty() {
            Some(default_command_entrypoint(parsed, allow_placeholder_image)?)
        } else {
            None
        };
        let command_template = if parsed.command_template.is_empty() {
            vec![default_entrypoint.clone().unwrap_or_else(|| parsed.tool_id.clone())]
        } else {
            parsed.command_template.clone()
        };
        let image = match parsed.container.as_ref() {
            Some(container) => ContainerImageRefV1 {
                image: container.image.clone(),
                digest: container.digest.clone(),
            },
            None => {
                let image = if allow_placeholder_image {
                    parsed.tool_id.clone()
                } else {
                    default_entrypoint.clone().unwrap_or_else(|| parsed.tool_id.clone())
                };
                ContainerImageRefV1 { image, digest: None }
            }
        };

        Ok(ToolExecutionSpecV1 {
            tool_id: tool_id.clone(),
            tool_version: parsed.default_version.clone(),
            image,
            command: CommandSpecV1 { template: command_template },
            resources: parsed.constraints.clone().unwrap_or_default(),
        })
    }
}

#[cfg(test)]
#[path = "../../tests/contracts/selection/tool_definitions.rs"]
mod tests;
