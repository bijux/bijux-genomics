use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, StageId, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastqDomainToolSupportLevel {
    Supported,
    Planned,
}

impl FastqDomainToolSupportLevel {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Planned => "planned",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FastqDomainToolContractMetadata {
    pub tool_id: ToolId,
    pub support_level: FastqDomainToolSupportLevel,
    pub stage_ids: Vec<StageId>,
    pub planned_stage_ids: Vec<StageId>,
}

impl FastqDomainToolContractMetadata {
    #[must_use]
    pub fn pair_support_level(&self, stage_id: &StageId) -> FastqDomainToolSupportLevel {
        if self.planned_stage_ids.iter().any(|candidate| candidate == stage_id)
            || self.support_level == FastqDomainToolSupportLevel::Planned
        {
            FastqDomainToolSupportLevel::Planned
        } else {
            FastqDomainToolSupportLevel::Supported
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
/// Returns an error if the governed FASTQ domain tool YAML cannot be read or omits required
/// support metadata.
pub fn load_fastq_domain_tool_contract_metadata(
    repo_root: &Path,
    tool_id: &ToolId,
) -> Result<FastqDomainToolContractMetadata> {
    let parsed = load_domain_tool_yaml(repo_root, tool_id)?;
    let support_level = match parsed.status.as_str() {
        "supported" => FastqDomainToolSupportLevel::Supported,
        "planned" => FastqDomainToolSupportLevel::Planned,
        other => {
            return Err(anyhow!(
                "governed FASTQ tool yaml {} declares unsupported status `{other}`",
                tool_id.as_str()
            ))
        }
    };

    let stage_ids = parsed.stage_ids.iter().cloned().map(StageId::new).collect::<Vec<_>>();
    let planned_stage_ids =
        parsed.planned_stage_ids.iter().cloned().map(StageId::new).collect::<Vec<_>>();

    Ok(FastqDomainToolContractMetadata {
        tool_id: tool_id.clone(),
        support_level,
        stage_ids,
        planned_stage_ids,
    })
}

/// # Errors
/// Returns an error if the governed FASTQ domain tool YAML cannot be read, does not match the
/// requested stage/tool pair, or omits required execution-spec fields.
pub fn load_fastq_domain_tool_execution_spec(
    repo_root: &Path,
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Result<ToolExecutionSpecV1> {
    let parsed = load_domain_tool_yaml(repo_root, tool_id)?;
    let yaml_path =
        repo_root.join("domain").join("fastq").join("tools").join(format!("{tool_id}.yaml"));

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
        Some(default_command_entrypoint(&parsed)?)
    } else {
        None
    };
    let command_template = if parsed.command_template.is_empty() {
        vec![default_entrypoint.clone().unwrap_or_else(|| parsed.tool_id.clone())]
    } else {
        parsed.command_template.clone()
    };
    let image = match parsed.container {
        Some(container) => ContainerImageRefV1 { image: container.image, digest: container.digest },
        None => ContainerImageRefV1 {
            image: default_entrypoint.unwrap_or_else(|| parsed.tool_id.clone()),
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

fn load_domain_tool_yaml(repo_root: &Path, tool_id: &ToolId) -> Result<DomainToolYaml> {
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

    Ok(parsed)
}

fn default_command_entrypoint(parsed: &DomainToolYaml) -> Result<String> {
    let install_kind = parsed.install_kind.as_deref().unwrap_or("container");
    if install_kind == "workspace_binary" {
        return workspace_binary_entrypoint(parsed);
    }
    if parsed.container.is_none() {
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

#[cfg(test)]
mod tests {
    use super::{
        load_fastq_domain_tool_contract_metadata, load_fastq_domain_tool_execution_spec,
        FastqDomainToolSupportLevel,
    };
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
    fn load_fastq_domain_tool_execution_spec_accepts_supported_workspace_binary_stage() -> Result<()>
    {
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

    #[test]
    fn load_fastq_domain_tool_contract_metadata_reads_supported_stage_status() -> Result<()> {
        let repo_root = repo_root();
        let tool_id = ToolId::new("krakenuniq");

        let metadata = load_fastq_domain_tool_contract_metadata(&repo_root, &tool_id)?;

        assert_eq!(metadata.tool_id.as_str(), "krakenuniq");
        assert_eq!(metadata.support_level, FastqDomainToolSupportLevel::Supported);
        assert!(
            metadata.stage_ids.iter().any(|stage_id| stage_id.as_str() == "fastq.screen_taxonomy"),
            "krakenuniq metadata must retain direct FASTQ stage admissions"
        );
        assert_eq!(
            metadata
                .pair_support_level(&StageId::new("fastq.screen_taxonomy".to_string()))
                .as_str(),
            "supported"
        );
        Ok(())
    }

    #[test]
    fn load_fastq_domain_tool_contract_metadata_reads_planned_tool_status() -> Result<()> {
        let repo_root = repo_root();
        let tool_id = ToolId::new("seqpurge");

        let metadata = load_fastq_domain_tool_contract_metadata(&repo_root, &tool_id)?;

        assert_eq!(metadata.tool_id.as_str(), "seqpurge");
        assert_eq!(metadata.support_level, FastqDomainToolSupportLevel::Planned);
        assert!(
            metadata.stage_ids.iter().any(|stage_id| stage_id.as_str() == "fastq.trim_reads"),
            "seqpurge metadata must retain admitted FASTQ stages"
        );
        assert_eq!(
            metadata.pair_support_level(&StageId::new("fastq.trim_reads".to_string())).as_str(),
            "planned"
        );
        Ok(())
    }

    #[test]
    fn load_fastq_domain_tool_contract_metadata_keeps_seqfu_on_supported_profile_routes(
    ) -> Result<()> {
        let repo_root = repo_root();
        let tool_id = ToolId::new("seqfu");
        let metadata = load_fastq_domain_tool_contract_metadata(&repo_root, &tool_id)?;

        assert_eq!(metadata.support_level, FastqDomainToolSupportLevel::Supported);
        assert!(
            metadata.stage_ids.iter().any(|stage_id| stage_id.as_str() == "fastq.profile_reads"),
            "seqfu must retain governed profile-reads admission"
        );
        assert!(
            metadata.planned_stage_ids.is_empty(),
            "seqfu must not retain any planned stage admission once normalize-abundance is removed"
        );
        assert_eq!(
            metadata.pair_support_level(&StageId::new("fastq.profile_reads".to_string())).as_str(),
            "supported"
        );
        Ok(())
    }

    #[test]
    fn load_fastq_domain_tool_execution_spec_accepts_supported_container_stage() -> Result<()> {
        let repo_root = repo_root();
        let stage_id = StageId::new("fastq.trim_terminal_damage".to_string());
        let tool_id = ToolId::new("cutadapt");

        let spec = load_fastq_domain_tool_execution_spec(&repo_root, &stage_id, &tool_id)?;

        assert_eq!(spec.tool_id.as_str(), "cutadapt");
        assert_eq!(spec.command.template[0], "cutadapt".to_string());
        assert_eq!(spec.image.image, "bijuxdna/cutadapt");
        assert!(spec.image.digest.is_some());
        Ok(())
    }

    #[test]
    fn load_fastq_domain_tool_execution_spec_accepts_container_stage_without_command_template(
    ) -> Result<()> {
        let repo_root = repo_root();
        let stage_id = StageId::new("fastq.deplete_host".to_string());
        let tool_id = ToolId::new("bowtie2");

        let spec = load_fastq_domain_tool_execution_spec(&repo_root, &stage_id, &tool_id)?;

        assert_eq!(spec.tool_id.as_str(), "bowtie2");
        assert_eq!(spec.command.template, vec!["bowtie2".to_string()]);
        assert_eq!(spec.image.image, "bijuxdna/bowtie2");
        assert!(spec.image.digest.is_none());
        Ok(())
    }

    #[test]
    fn load_fastq_domain_tool_execution_spec_accepts_seqfu_profile_stage() -> Result<()> {
        let repo_root = repo_root();
        let stage_id = StageId::new("fastq.profile_read_lengths".to_string());
        let tool_id = ToolId::new("seqfu");

        let spec = load_fastq_domain_tool_execution_spec(&repo_root, &stage_id, &tool_id)?;

        assert_eq!(spec.tool_id.as_str(), "seqfu");
        assert_eq!(spec.command.template, vec!["seqfu".to_string()]);
        assert_eq!(spec.image.image, "bijuxdna/seqfu:2.4.0");
        assert!(spec.image.digest.is_none());
        Ok(())
    }
}
