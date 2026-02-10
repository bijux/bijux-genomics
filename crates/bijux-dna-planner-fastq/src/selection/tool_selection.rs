use std::collections::BTreeSet;

use bijux_dna_core::ids::{id_catalog, StageId, ToolId};

fn registry_toml() -> Option<toml::Value> {
    let cwd = std::env::current_dir().ok()?;
    let path = cwd.join("configs").join("tool_registry.toml");
    let raw = std::fs::read_to_string(path).ok()?;
    raw.parse::<toml::Value>().ok()
}

#[must_use]
pub fn allowed_tools_for_stage(stage_id: &StageId) -> Vec<ToolId> {
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
            .any(|stage| stage == stage_id.as_str())
        {
            tools.insert(ToolId::new(tool_id.to_string()));
        }
    }
    let mut tools = tools.into_iter().collect::<Vec<_>>();
    tools.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    tools
}

#[must_use]
pub fn default_tool_for_stage(stage_id: &StageId) -> Option<ToolId> {
    if stage_id.as_str() == id_catalog::FASTQ_PREPROCESS {
        return Some(ToolId::from_static(id_catalog::TOOL_PLANNER));
    }
    let parsed = registry_toml()?;
    let stages = parsed.get("stages")?.as_array()?;
    for stage in stages {
        let id = stage.get("id").and_then(toml::Value::as_str)?;
        if id != stage_id.as_str() {
            continue;
        }
        let primary = stage
            .get("primary_tools")
            .and_then(toml::Value::as_array)
            .cloned()
            .unwrap_or_default();
        if let Some(tool) = primary.first().and_then(toml::Value::as_str) {
            return Some(ToolId::new(tool.to_string()));
        }
    }
    allowed_tools_for_stage(stage_id).first().cloned()
}
