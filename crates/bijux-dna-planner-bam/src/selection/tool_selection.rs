use std::collections::BTreeSet;

use bijux_dna_core::ids::ToolId;
use bijux_dna_domain_bam::BamStage;

fn registry_toml() -> Option<toml::Value> {
    let cwd = std::env::current_dir().ok()?;
    let path = cwd.join("configs").join("tool_registry.toml");
    let raw = std::fs::read_to_string(path).ok()?;
    raw.parse::<toml::Value>().ok()
}

#[must_use]
pub fn allowed_tools_for_stage(stage: BamStage) -> Vec<String> {
    canonical_tools_for_stage(stage)
}

#[must_use]
#[allow(dead_code)]
pub fn default_tool_for_stage(stage: BamStage) -> String {
    default_tool(stage).to_string()
}

#[must_use]
pub fn canonical_tools_for_stage(stage: BamStage) -> Vec<String> {
    let mut tools = BTreeSet::new();
    let Some(parsed) = registry_toml() else {
        return Vec::new();
    };
    let Some(entries) = parsed.get("tools").and_then(toml::Value::as_array) else {
        return Vec::new();
    };
    for tool in entries {
        let Some(tool_id) = tool.get("id").and_then(toml::Value::as_str) else {
            continue;
        };
        let stage_ids = tool
            .get("stage_ids")
            .and_then(toml::Value::as_array)
            .cloned()
            .unwrap_or_default();
        if stage_ids
            .iter()
            .filter_map(toml::Value::as_str)
            .any(|id| id == stage.as_str())
        {
            tools.insert(tool_id.to_string());
        }
    }
    let mut tools = tools.into_iter().collect::<Vec<_>>();
    tools.sort();
    tools
}

#[must_use]
pub fn default_tool(stage: BamStage) -> ToolId {
    if let Some(parsed) = registry_toml() {
        if let Some(stages) = parsed.get("stages").and_then(toml::Value::as_array) {
            for stage_entry in stages {
                let Some(id) = stage_entry.get("id").and_then(toml::Value::as_str) else {
                    continue;
                };
                if id != stage.as_str() {
                    continue;
                }
                let primary = stage_entry
                    .get("primary_tools")
                    .and_then(toml::Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                if let Some(tool) = primary.first().and_then(toml::Value::as_str) {
                    return ToolId::new(tool.to_string());
                }
            }
        }
    }
    canonical_tools_for_stage(stage)
        .first()
        .cloned()
        .map_or_else(
            || panic!("no compatible tool found for stage {}", stage.as_str()),
            ToolId::new,
        )
}
