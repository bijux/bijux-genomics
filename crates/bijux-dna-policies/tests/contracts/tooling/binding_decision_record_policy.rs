#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;

fn list(table: &toml::Value, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[test]
fn policy__contracts__binding_decision_record_policy__complex_binding_changes_require_decision_record(
) {
    let root = support::workspace_root();
    let registry_raw = std::fs::read_to_string(root.join("configs/ci/tool_registry.toml"))
        .expect("read configs/ci/tool_registry.toml");
    let registry: toml::Value = registry_raw
        .parse()
        .expect("parse configs/ci/tool_registry.toml");

    let decision_path = root.join("docs/decisions/TOOL_BINDING_DECISIONS.md");
    let decision_raw = std::fs::read_to_string(&decision_path)
        .unwrap_or_else(|_| panic!("read {}", decision_path.display()));

    let tools = registry
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut required_mentions = BTreeSet::new();
    for tool in tools {
        let Some(id) = tool.get("id").and_then(toml::Value::as_str) else {
            continue;
        };
        let mut bindings = list(&tool, "bindings");
        let stage_ids = list(&tool, "stage_ids");
        if bindings.is_empty() {
            bindings = stage_ids.clone();
        }
        if bindings.is_empty() {
            continue;
        }
        let domains = bindings
            .iter()
            .filter_map(|stage| stage.split('.').next())
            .collect::<BTreeSet<_>>();
        let changed_shape = domains.len() > 1 || bindings != stage_ids;
        if changed_shape {
            required_mentions.insert(id.to_string());
        }
    }

    let mut offenders = Vec::new();
    for tool_id in required_mentions {
        let needle = format!("- {tool_id}:");
        if !decision_raw.contains(&needle) {
            offenders.push(format!(
                "{} missing decision entry `{}`",
                decision_path.display(),
                needle
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "binding decision record policy violations:\n{}",
        offenders.join("\n")
    );
}
