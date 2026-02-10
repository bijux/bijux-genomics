use std::path::Path;

use anyhow::{anyhow, Context, Result};

use bijux_dna_core::contract::{
    Cardinality, PortSpec, StageId, StageSpec, ToolManifest, ToolRegistry,
};
use bijux_dna_core::ids::ToolId;

fn list_strings(table: &toml::Value, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

/// # Errors
/// Returns an error if registry config cannot be read or parsed.
pub fn load_manifests(registry_path: &Path) -> Result<ToolRegistry> {
    let mut registry = ToolRegistry::default();
    let registry_path = if registry_path.is_dir() {
        if registry_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "domain")
        {
            registry_path
                .parent()
                .unwrap_or(registry_path)
                .join("configs")
                .join("tool_registry.toml")
        } else {
            registry_path.join("configs").join("tool_registry.toml")
        }
    } else {
        registry_path.to_path_buf()
    };
    if !registry_path.exists() {
        return Err(anyhow!(
            "registry file {} does not exist",
            registry_path.display()
        ));
    }
    let raw = std::fs::read_to_string(&registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let parsed: toml::Value = raw
        .parse()
        .with_context(|| format!("parse {}", registry_path.display()))?;

    for stage in parsed
        .get("stages")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let Some(stage_id_raw) = stage.get("id").and_then(toml::Value::as_str) else {
            continue;
        };
        let stage_id = StageId::try_from(stage_id_raw)
            .map_err(|err| anyhow!("invalid stage id `{stage_id_raw}`: {err}"))?;
        let spec = StageSpec {
            stage_id,
            inputs: vec![PortSpec {
                name: "input".to_string(),
                data_type: "unknown".to_string(),
                cardinality: Cardinality::Many,
            }],
            outputs: vec![PortSpec {
                name: "output".to_string(),
                data_type: "unknown".to_string(),
                cardinality: Cardinality::Many,
            }],
            parameters: Vec::new(),
            metrics: Vec::new(),
            description: Some("generated from configs/tool_registry.toml".to_string()),
            mutates_fastq: false,
            report_only: false,
            may_change_read_count: false,
            image_requirements: None,
            extends: None,
        };
        registry.insert_stage(spec);
    }

    for tool in parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let Some(tool_id_raw) = tool.get("id").and_then(toml::Value::as_str) else {
            continue;
        };
        for stage_id_raw in list_strings(&tool, "stage_ids") {
            let tool_id = ToolId::try_from(tool_id_raw)
                .map_err(|err| anyhow!("invalid tool id `{tool_id_raw}`: {err}"))?;
            let stage_id = StageId::try_from(stage_id_raw.as_str())
                .map_err(|err| anyhow!("invalid stage id `{stage_id_raw}`: {err}"))?;
            registry.insert_tool(ToolManifest {
                tool_id,
                stage_id,
                role: Default::default(),
                command_template: Vec::new(),
                outputs: Vec::new(),
                metrics_parser: None,
                constraints: Default::default(),
                execution_contract: Default::default(),
            });
        }
    }
    registry.sort_tools_for_determinism();
    Ok(registry)
}
