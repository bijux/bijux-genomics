#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::collections::BTreeMap;
use support::workspace_root;

#[test]
fn policy__contracts__stage_registry_fixture_completeness__each_stage_has_tool_metrics_and_smoke_contract(
) {
    let registry_path = workspace_root().join("configs/tool_registry.toml");
    let raw = std::fs::read_to_string(&registry_path).expect("read configs/tool_registry.toml");
    let parsed: toml::Value = raw.parse().expect("parse configs/tool_registry.toml");

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

    let mut by_tool_id: BTreeMap<String, toml::Value> = BTreeMap::new();
    for tool in tools {
        if let Some(id) = tool.get("id").and_then(toml::Value::as_str) {
            by_tool_id.insert(id.to_string(), tool);
        }
    }

    let mut offenders = Vec::new();
    for stage in stages {
        let Some(stage_id) = stage.get("id").and_then(toml::Value::as_str) else {
            offenders.push("stage entry missing id".to_string());
            continue;
        };

        let stage_tool_ids = stage_tools_from_matrix(&stage);
        if stage_tool_ids.is_empty() {
            offenders.push(format!("stage={stage_id}: no tools mapped in stage matrix"));
            continue;
        }
        let stage_tools = stage_tool_ids
            .iter()
            .filter_map(|id| by_tool_id.get(id))
            .collect::<Vec<_>>();
        if stage_tools.len() != stage_tool_ids.len() {
            offenders.push(format!(
                "stage={stage_id}: one or more stage matrix tools are missing from [[tools]]"
            ));
        }

        let has_metrics_schema = stage_tools.iter().any(|tool| {
            tool.get("metrics_schema")
                .and_then(toml::Value::as_str)
                .is_some_and(|v| !v.trim().is_empty())
        });
        if !has_metrics_schema {
            offenders.push(format!(
                "stage={stage_id}: missing metrics_schema across mapped tools"
            ));
        }

        let has_smoke = stage_tools.iter().any(|tool| {
            let version = tool
                .get("smoke_version_cmd")
                .and_then(toml::Value::as_str)
                .unwrap_or("")
                .trim();
            let help = tool
                .get("smoke_help_cmd")
                .and_then(toml::Value::as_str)
                .unwrap_or("")
                .trim();
            !version.is_empty() && !help.is_empty()
        });
        if !has_smoke {
            offenders.push(format!(
                "stage={stage_id}: missing smoke commands across mapped tools"
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "stage registry fixture completeness violations:\n{}",
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

fn stage_tools_from_matrix(stage: &toml::Value) -> Vec<String> {
    let mut out = Vec::new();
    for key in [
        "primary_tools",
        "optional_alternatives",
        "validation_tools",
        "reporting_tools",
    ] {
        out.extend(list(stage, key));
    }
    out.sort();
    out.dedup();
    out
}
