use std::path::Path;

use anyhow::{Context, Result};
use bijux_dna_core::contract::{
    BackendVersionPolicy, ExecutionContract, StageCapabilitySpec, ToolConstraints, ToolManifest,
    ToolRegistry, ToolRole,
};
use bijux_dna_core::ids::ToolId;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
struct DomainToolYaml {
    tool_id: String,
    #[serde(default)]
    stage_id: Option<String>,
    #[serde(default)]
    stage_ids: Vec<String>,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    command_template: Vec<String>,
    #[serde(default)]
    outputs: Vec<bijux_dna_core::contract::PortSpec>,
    #[serde(default)]
    metrics_parser: Option<String>,
    #[serde(default)]
    constraints: Option<ToolConstraints>,
    #[serde(default)]
    execution_contract: Option<ExecutionContract>,
}

fn parse_tool_role(raw: Option<&str>) -> ToolRole {
    match raw {
        Some("diagnostic") => ToolRole::Diagnostic,
        Some("experimental") => ToolRole::Experimental,
        _ => ToolRole::Authoritative,
    }
}

/// # Errors
/// Returns an error if the workspace domain tool registry cannot be read.
pub fn load_domain_tool_registry(workspace_root: &Path) -> Result<ToolRegistry> {
    let mut registry = ToolRegistry::default();
    for domain_name in ["fastq", "bam"] {
        let tools_dir = workspace_root.join("domain").join(domain_name).join("tools");
        if !tools_dir.exists() {
            continue;
        }
        for entry in std::fs::read_dir(&tools_dir)
            .with_context(|| format!("read {}", tools_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
                continue;
            }
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with('_'))
            {
                continue;
            }
            let raw = std::fs::read_to_string(&path)
                .with_context(|| format!("read {}", path.display()))?;
            let tool: DomainToolYaml = bijux_dna_infra::formats::parse_yaml(&raw)
                .with_context(|| format!("parse {}", path.display()))?;
            let tool_id = ToolId::try_from(tool.tool_id.as_str())
                .with_context(|| format!("invalid tool id in {}", path.display()))?;
            let mut stage_ids = Vec::new();
            if let Some(stage_id) = tool.stage_id {
                stage_ids.push(stage_id);
            }
            stage_ids.extend(tool.stage_ids);
            for stage_id_raw in stage_ids {
                let stage_id = bijux_dna_core::prelude::StageId::try_from(stage_id_raw.as_str())
                    .with_context(|| format!("invalid stage id in {}", path.display()))?;
                registry.insert_tool(ToolManifest {
                    tool_id: tool_id.clone(),
                    stage_id,
                    role: parse_tool_role(tool.role.as_deref()),
                    command_template: tool.command_template.clone(),
                    outputs: tool.outputs.clone(),
                    metrics_parser: tool.metrics_parser.clone(),
                    constraints: tool.constraints.clone().unwrap_or_default(),
                    execution_contract: tool.execution_contract.clone().unwrap_or_default(),
                    supported_modes: Vec::new(),
                    backend_version_policy: BackendVersionPolicy::default(),
                    capability_contract: StageCapabilitySpec::default(),
                });
            }
        }
    }
    registry.sort_tools_for_determinism();
    Ok(registry)
}
