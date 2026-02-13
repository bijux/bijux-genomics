#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;
use support::workspace_root;

#[test]
fn policy__contracts__stage_tool_matrix_policy__stages_have_primary_validation_and_reporting_contracts(
) {
    let registry_path = workspace_root().join("configs/ci/tool_registry.toml");
    let raw = std::fs::read_to_string(&registry_path).expect("read configs/ci/tool_registry.toml");
    let parsed: toml::Value = raw.parse().expect("parse configs/ci/tool_registry.toml");

    let mut offenders = Vec::new();

    let tools = parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let tool_ids: BTreeSet<String> = tools
        .iter()
        .filter_map(|t| t.get("id").and_then(toml::Value::as_str))
        .map(str::to_string)
        .collect();

    let stages = parsed
        .get("stages")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();
    if stages.is_empty() {
        offenders.push("missing [[stages]] matrix in configs/ci/tool_registry.toml".to_string());
    }

    for stage in &stages {
        let Some(id) = stage.get("id").and_then(toml::Value::as_str) else {
            offenders.push("stage entry missing id".to_string());
            continue;
        };

        let primary = list(stage, "primary_tools");
        let validation = list(stage, "validation_tools");
        let reporting = list(stage, "reporting_tools");
        let _optional = list(stage, "optional_alternatives");
        let _planned = list(stage, "planned_out_of_scope");

        if primary.is_empty() {
            offenders.push(format!(
                "stage={id}: primary_tools must have at least one tool"
            ));
        }

        let requires_validation = stage
            .get("requires_validation")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);
        if requires_validation && validation.is_empty() {
            offenders.push(format!(
                "stage={id}: requires_validation=true but validation_tools is empty"
            ));
        }

        let requires_reporting = stage
            .get("requires_reporting")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);
        if requires_reporting && reporting.is_empty() {
            offenders.push(format!(
                "stage={id}: requires_reporting=true but reporting_tools is empty"
            ));
        }

        if id.contains("qc") && !reporting.iter().any(|t| t == "multiqc") {
            offenders.push(format!(
                "stage={id}: qc stages must include multiqc in reporting_tools"
            ));
        }

        for tool in primary
            .iter()
            .chain(validation.iter())
            .chain(reporting.iter())
        {
            if !tool_ids.contains(tool) {
                offenders.push(format!("stage={id}: unknown tool id `{tool}` in matrix"));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "stage tool matrix policy violations:\n{}",
        offenders.join("\n")
    );
}

fn list(stage: &toml::Value, key: &str) -> Vec<String> {
    stage
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
