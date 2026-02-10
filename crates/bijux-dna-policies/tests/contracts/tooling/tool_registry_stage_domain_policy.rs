#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::collections::{BTreeMap, BTreeSet};
use support::workspace_root;

#[test]
fn policy__contracts__tool_registry_stage_domain_policy__each_tool_has_exactly_one_domain_and_stage_binding(
) {
    let registry_path = workspace_root().join("configs/tools.toml");
    let raw = std::fs::read_to_string(&registry_path).expect("read configs/tools.toml");
    let parsed: toml::Value = raw.parse().expect("parse configs/tools.toml");

    let tools = parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let stages = parsed
        .get("stages")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut tool_stage_refs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for stage in &stages {
        let Some(stage_id) = stage.get("id").and_then(toml::Value::as_str) else {
            continue;
        };
        for key in [
            "primary_tools",
            "optional_alternatives",
            "validation_tools",
            "reporting_tools",
        ] {
            for tool in list(stage, key) {
                tool_stage_refs
                    .entry(tool)
                    .or_default()
                    .insert(stage_id.to_string());
            }
        }
    }

    let mut offenders = Vec::new();
    for tool in &tools {
        let id = tool
            .get("id")
            .and_then(toml::Value::as_str)
            .unwrap_or("<missing-id>");
        let declared_domain = tool
            .get("domain")
            .and_then(toml::Value::as_str)
            .unwrap_or("")
            .to_string();
        let declared_stage_ids = list(tool, "stage_ids");
        let discovered_stage_ids = tool_stage_refs
            .get(id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>();

        if declared_domain.is_empty() {
            offenders.push(format!("tool={id}: missing `domain`"));
        }
        if declared_stage_ids.is_empty() {
            offenders.push(format!("tool={id}: missing non-empty `stage_ids`"));
        }
        if discovered_stage_ids.is_empty() {
            offenders.push(format!(
                "tool={id}: referenced by no stage; every tool must appear in at least one stage"
            ));
            continue;
        }

        let stage_domain_set = discovered_stage_ids
            .iter()
            .map(|stage_id| {
                stage_id
                    .split('.')
                    .next()
                    .map(str::to_string)
                    .unwrap_or_else(|| "unknown".to_string())
            })
            .collect::<BTreeSet<_>>();

        if stage_domain_set.len() != 1 {
            offenders.push(format!(
                "tool={id}: must map to exactly one domain; found {:?}",
                stage_domain_set
            ));
        } else if let Some(actual_domain) = stage_domain_set.iter().next() {
            if *actual_domain != declared_domain {
                offenders.push(format!(
                    "tool={id}: declared domain `{declared_domain}` does not match discovered `{actual_domain}`"
                ));
            }
        }

        for stage_id in &declared_stage_ids {
            if !discovered_stage_ids.iter().any(|s| s == stage_id) {
                offenders.push(format!(
                    "tool={id}: declared stage `{stage_id}` not present in stage matrix references"
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "tool domain/stage binding policy violations:\n{}",
        offenders.join("\n")
    );
}

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
