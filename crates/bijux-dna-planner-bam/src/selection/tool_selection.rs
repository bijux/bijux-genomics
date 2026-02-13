use std::collections::BTreeSet;
use std::path::PathBuf;

use bijux_dna_core::ids::ToolId;
use bijux_dna_domain_bam::BamStage;

fn registry_toml() -> Option<toml::Value> {
    let cwd = std::env::current_dir().ok()?;
    let mut candidates = vec![bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml")];
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    candidates.push(
        manifest_dir
            .parent()
            .and_then(std::path::Path::parent)
            .map(|root| bijux_dna_infra::configs_file(&root, "ci/registry/tool_registry.toml"))?,
    );
    let path = candidates
        .into_iter()
        .find(|candidate| candidate.exists())?;
    let raw = std::fs::read_to_string(path).ok()?;
    raw.parse::<toml::Value>().ok()
}

#[must_use]
pub fn allowed_tools_for_stage(stage: BamStage) -> Vec<ToolId> {
    canonical_tools_for_stage(stage)
}

#[must_use]
#[allow(dead_code)]
pub fn default_tool_for_stage(stage: BamStage) -> ToolId {
    default_tool(stage)
}

#[must_use]
pub fn canonical_tools_for_stage(stage: BamStage) -> Vec<ToolId> {
    let mut tools = BTreeSet::new();
    let Some(parsed) = registry_toml() else {
        return domain_tools_for_stage(stage);
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
            tools.insert(ToolId::new(tool_id.to_string()));
        }
    }
    let mut tools = tools.into_iter().collect::<Vec<_>>();
    tools.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    if tools.is_empty() {
        domain_tools_for_stage(stage)
    } else {
        tools
    }
}

#[must_use]
pub fn default_tool(stage: BamStage) -> ToolId {
    let allowed = canonical_tools_for_stage(stage);
    if allowed.is_empty() {
        panic!(
            "no configured tool candidates for stage {}; planner must not silently fallback",
            stage.as_str()
        );
    }
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
                    if allowed.iter().any(|candidate| candidate.as_str() == tool) {
                        return ToolId::new(tool.to_string());
                    }
                }
            }
        }
    }
    allowed
        .first()
        .cloned()
        .unwrap_or_else(|| panic!("no compatible tool found for stage {}", stage.as_str()))
}

fn domain_tools_for_stage(stage: BamStage) -> Vec<ToolId> {
    bijux_dna_domain_bam::stage_contract_json(stage.as_str())
        .and_then(|json| json.get("tool_ids").cloned())
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(|tool| ToolId::new(tool.to_string())))
        .collect()
}
