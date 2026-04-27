#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::{BTreeMap, BTreeSet};

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
fn policy__contracts__tool_role_capability_policy__stage_tools_match_required_roles() {
    let root = support::workspace_root();
    let raw = std::fs::read_to_string(root.join("configs/ci/registry/tool_registry.toml"))
        .expect("read registry");
    let parsed: toml::Value = raw.parse().expect("parse registry");

    let tools = parsed.get("tools").and_then(toml::Value::as_array).cloned().unwrap_or_default();
    let stages = parsed.get("stages").and_then(toml::Value::as_array).cloned().unwrap_or_default();

    let tool_roles = tools
        .iter()
        .filter_map(|tool| {
            let id = tool.get("id").and_then(toml::Value::as_str)?;
            let role = tool.get("tool_role").and_then(toml::Value::as_str)?;
            Some((id.to_string(), role.to_string()))
        })
        .collect::<BTreeMap<_, _>>();

    let mut offenders = Vec::new();
    for stage in &stages {
        let Some(stage_id) = stage.get("id").and_then(toml::Value::as_str) else {
            continue;
        };
        let required = list(stage, "required_tool_roles").into_iter().collect::<BTreeSet<_>>();
        if required.is_empty() {
            offenders.push(format!("stage={stage_id}: missing required_tool_roles"));
            continue;
        }
        let mut stage_tools = BTreeSet::new();
        for key in ["primary_tools", "optional_alternatives", "validation_tools", "reporting_tools"]
        {
            stage_tools.extend(list(stage, key));
        }
        for tool in stage_tools {
            let Some(role) = tool_roles.get(&tool) else {
                offenders.push(format!("stage={stage_id}: tool={tool} missing tool_role"));
                continue;
            };
            if !required.contains(role) {
                offenders.push(format!(
                    "stage={stage_id}: tool={tool} has tool_role={role} not in required_tool_roles={required:?}"
                ));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "tool role/stage role drift:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__tool_role_capability_policy__fastq_benchmark_stages_capabilities_satisfied() {
    let root = support::workspace_root();
    let index_raw =
        std::fs::read_to_string(root.join("domain/fastq/index.yaml")).expect("read fastq index");

    let benchmark_stages = [
        "fastq.validate_reads",
        "fastq.trim_reads",
        "fastq.filter_reads",
        "fastq.profile_reads",
        "fastq.report_qc",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();

    let mut matrix = BTreeMap::<String, Vec<String>>::new();
    let mut in_matrix = false;
    for line in index_raw.lines() {
        if line.trim() == "stage_tool_compatibility:" {
            in_matrix = true;
            continue;
        }
        if in_matrix && !line.starts_with("  ") {
            in_matrix = false;
        }
        if !in_matrix {
            continue;
        }
        let trimmed = line.trim();
        if !trimmed.contains(':') || !trimmed.contains('[') {
            continue;
        }
        let mut parts = trimmed.splitn(2, ':');
        let Some(stage_id) = parts.next().map(str::trim) else {
            continue;
        };
        let Some(rhs) = parts.next().map(str::trim) else {
            continue;
        };
        let tools = rhs
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        matrix.insert(stage_id.to_string(), tools);
    }

    let mut capability_by_tool = BTreeMap::<String, BTreeSet<String>>::new();
    let mut required_by_stage = BTreeMap::<String, BTreeSet<String>>::new();
    for stage_id in &benchmark_stages {
        let stage_suffix = stage_id.split_once('.').map_or(*stage_id, |(_, rhs)| rhs);
        let stage_path =
            root.join(format!("domain/fastq/stages/{}.yaml", stage_suffix.replace('.', "_")));
        let stage_raw = std::fs::read_to_string(&stage_path)
            .unwrap_or_else(|_| panic!("read {}", stage_path.display()));
        let mut req = BTreeSet::new();
        let mut in_req = false;
        for line in stage_raw.lines() {
            if line.trim() == "tool_capability_requirements:" {
                in_req = true;
                continue;
            }
            if in_req && !line.starts_with("  ") {
                in_req = false;
            }
            if !in_req {
                continue;
            }
            if let Some(cap) = line.trim().strip_prefix("- ") {
                req.insert(cap.trim().trim_matches('"').to_string());
            }
        }
        required_by_stage.insert((*stage_id).to_string(), req);
    }

    let tools_dir = root.join("domain/fastq/tools");
    for entry in std::fs::read_dir(&tools_dir).expect("read tools dir") {
        let path = entry.expect("tool entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        let raw = std::fs::read_to_string(&path).expect("read tool yaml");
        let tool_id = raw.lines().find_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("tool_id:") {
                return None;
            }
            Some(trimmed.trim_start_matches("tool_id:").trim().trim_matches('"').to_string())
        });
        let Some(tool_id) = tool_id else {
            continue;
        };
        let mut caps = BTreeSet::new();
        let mut in_caps = false;
        for line in raw.lines() {
            if line.trim() == "capabilities:" {
                in_caps = true;
                continue;
            }
            if in_caps && !line.starts_with("  ") {
                in_caps = false;
            }
            if !in_caps {
                continue;
            }
            if let Some(cap) = line.trim().strip_prefix("- ") {
                caps.insert(cap.trim().trim_matches('"').to_string());
            }
        }
        capability_by_tool.insert(tool_id, caps);
    }

    let mut offenders = Vec::new();
    for stage_id in &benchmark_stages {
        let required = required_by_stage.get(*stage_id).cloned().unwrap_or_default();
        for tool in matrix.get(*stage_id).cloned().unwrap_or_default() {
            let caps = capability_by_tool.get(&tool).cloned().unwrap_or_default();
            for req in &required {
                if !caps.contains(req) {
                    offenders.push(format!(
                        "stage={stage_id} tool={tool} missing required capability `{req}`"
                    ));
                }
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "fastq benchmark capability policy violations:\n{}",
        offenders.join("\n")
    );
}
