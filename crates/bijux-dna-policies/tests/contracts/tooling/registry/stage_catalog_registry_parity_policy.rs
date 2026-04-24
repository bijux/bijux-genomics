#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::{BTreeMap, BTreeSet};

fn table_array<'a>(root: &'a toml::Value, key: &str) -> Vec<&'a toml::Value> {
    root.get(key)
        .and_then(toml::Value::as_array)
        .map(|rows| rows.iter().collect())
        .unwrap_or_default()
}

fn list(table: &toml::Value, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|values| {
            values.iter().filter_map(toml::Value::as_str).map(str::to_string).collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[test]
fn policy__contracts__stage_catalog_registry_parity_policy__supported_stage_tool_lists_match_production_registry(
) {
    let root = support::workspace_root();
    let stages_raw = std::fs::read_to_string(root.join("configs/ci/stages/stages.toml"))
        .expect("read configs/ci/stages/stages.toml");
    let registry_raw = std::fs::read_to_string(root.join("configs/ci/registry/tool_registry.toml"))
        .expect("read configs/ci/registry/tool_registry.toml");

    let stages: toml::Value = stages_raw.parse().expect("parse stages.toml");
    let registry: toml::Value = registry_raw.parse().expect("parse tool_registry.toml");

    let mut registry_stage_tools = BTreeMap::<String, BTreeSet<String>>::new();
    for tool in table_array(&registry, "tools") {
        let Some(tool_id) = tool.get("id").and_then(toml::Value::as_str) else {
            continue;
        };
        for stage_id in list(tool, "stage_ids") {
            registry_stage_tools.entry(stage_id).or_default().insert(tool_id.to_string());
        }
    }

    let mut offenders = Vec::new();
    for stage in table_array(&stages, "stages") {
        let Some(stage_id) = stage.get("id").and_then(toml::Value::as_str) else {
            continue;
        };
        let status = stage.get("status").and_then(toml::Value::as_str).unwrap_or("supported");
        if status != "supported" {
            continue;
        }
        let stage_tools = list(stage, "tools").into_iter().collect::<BTreeSet<_>>();
        let registry_tools = registry_stage_tools.get(stage_id).cloned().unwrap_or_default();
        if stage_tools != registry_tools {
            offenders.push(format!(
                "stage {stage_id} tools drifted: stages.toml={:?} tool_registry.toml={:?}",
                stage_tools, registry_tools
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "supported stage catalog drift detected:\n{}",
        offenders.join("\n")
    );
}
