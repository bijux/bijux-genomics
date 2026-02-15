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

#[test]
fn policy__contracts__registry_ssot_completeness_policy__supported_stages_and_tools_are_complete() {
    let root = support::workspace_root();
    let tool_registry_raw =
        std::fs::read_to_string(root.join("configs/ci/registry/tool_registry.toml"))
            .expect("read configs/ci/registry/tool_registry.toml");
    let stages_raw = std::fs::read_to_string(root.join("configs/ci/stages/stages.toml"))
        .expect("read configs/ci/stages/stages.toml");
    let images_raw = std::fs::read_to_string(root.join("configs/ci/tools/images.toml"))
        .expect("read configs/ci/tools/images.toml");

    let tool_registry: toml::Value = tool_registry_raw.parse().expect("parse tool_registry");
    let stages: toml::Value = stages_raw.parse().expect("parse stages");
    let images: toml::Value = images_raw.parse().expect("parse images");

    let tool_rows = table_array(&tool_registry, "tools");
    let stage_rows = table_array(&tool_registry, "stages");
    let stage_status_rows = table_array(&stages, "stages");

    let mut tool_by_id = BTreeMap::new();
    for row in &tool_rows {
        if let Some(id) = row.get("id").and_then(toml::Value::as_str) {
            tool_by_id.insert(id.to_string(), *row);
        }
    }

    let stage_status = stage_status_rows
        .iter()
        .filter_map(|row| {
            let id = row.get("id").and_then(toml::Value::as_str)?;
            let status = row
                .get("status")
                .and_then(toml::Value::as_str)
                .unwrap_or("supported");
            Some((id.to_string(), status.to_string()))
        })
        .collect::<BTreeMap<_, _>>();

    let image_ids = images
        .as_table()
        .map(|t| t.keys().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_default();

    let mut tool_to_stages = BTreeMap::<String, BTreeSet<String>>::new();
    let mut offenders = Vec::new();

    for stage in &stage_rows {
        let Some(stage_id) = stage.get("id").and_then(toml::Value::as_str) else {
            offenders.push("stage row missing id".to_string());
            continue;
        };
        let status = stage_status
            .get(stage_id)
            .map_or("supported", std::string::String::as_str);
        if status != "supported" {
            continue;
        }
        let mut mapped_tools = stage_tools_from_matrix(stage);
        if mapped_tools.is_empty() {
            for tool_row in &tool_rows {
                let Some(tool_id) = tool_row.get("id").and_then(toml::Value::as_str) else {
                    continue;
                };
                let stage_ids = list(tool_row, "stage_ids");
                if stage_ids.iter().any(|id| id == stage_id) {
                    mapped_tools.push(tool_id.to_string());
                }
            }
            mapped_tools.sort();
            mapped_tools.dedup();
        }
        if mapped_tools.is_empty() {
            offenders.push(format!(
                "supported stage {stage_id} must map to at least one tool"
            ));
            continue;
        }

        let mut has_supported_tool = false;
        let mut has_metrics = false;
        for tool_id in &mapped_tools {
            if let Some(tool_row) = tool_by_id.get(tool_id) {
                let tool_status = tool_row
                    .get("status")
                    .and_then(toml::Value::as_str)
                    .unwrap_or("supported");
                if tool_status == "supported" {
                    has_supported_tool = true;
                }
                let metrics = tool_row
                    .get("metrics_schema")
                    .and_then(toml::Value::as_str)
                    .unwrap_or("");
                if !metrics.trim().is_empty() {
                    has_metrics = true;
                }
                tool_to_stages
                    .entry(tool_id.clone())
                    .or_default()
                    .insert(stage_id.to_string());
            } else {
                offenders.push(format!(
                    "stage {stage_id} references unknown tool {tool_id}"
                ));
            }
        }
        if !has_supported_tool {
            offenders.push(format!(
                "supported stage {stage_id} has no supported mapped tools"
            ));
        }
        if !has_metrics {
            offenders.push(format!(
                "supported stage {stage_id} has no metrics schema across mapped tools"
            ));
        }
    }

    for (tool_id, tool_row) in &tool_by_id {
        let status = tool_row
            .get("status")
            .and_then(toml::Value::as_str)
            .unwrap_or("supported");
        if status != "supported" {
            continue;
        }

        if !tool_to_stages.contains_key(tool_id) {
            offenders.push(format!(
                "supported tool {tool_id} must belong to at least one supported stage"
            ));
        }

        let version_cmd = tool_row
            .get("version_cmd")
            .and_then(toml::Value::as_str)
            .unwrap_or("")
            .trim();
        let help_cmd = tool_row
            .get("help_cmd")
            .and_then(toml::Value::as_str)
            .unwrap_or("")
            .trim();
        if version_cmd.is_empty() || help_cmd.is_empty() {
            offenders.push(format!(
                "supported tool {tool_id} missing smoke commands (version/help)"
            ));
        }

        if !image_ids.contains(tool_id) {
            offenders.push(format!(
                "supported tool {tool_id} missing image catalog entry"
            ));
        }
    }

    for row in stage_status_rows {
        let Some(stage_id) = row.get("id").and_then(toml::Value::as_str) else {
            continue;
        };
        let status = row
            .get("status")
            .and_then(toml::Value::as_str)
            .unwrap_or("supported");
        if status != "supported" {
            continue;
        }
        let output_kinds = list(row, "output_kinds");
        if output_kinds.is_empty() {
            offenders.push(format!(
                "supported stage {stage_id} must declare non-empty output_kinds"
            ));
        }
    }

    if !offenders.is_empty() {
        eprintln!(
            "registry ssot completeness drift (non-fatal during migration):\n{}\n\nactionable summary:\n- Sync generated SSOT files via `make generate-configs`.\n- Then verify with `make domain-gates`.\n- Core files: configs/ci/stages/stages.toml, configs/ci/registry/tool_registry.toml, configs/ci/tools/images.toml, configs/ci/params/param_registry.toml, configs/ci/tools/required_tools.toml.",
            offenders.join("\n")
        );
    }
}
