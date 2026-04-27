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
fn policy__contracts__benchmark_suite_support_policy__production_benchmark_tools_must_appear_in_suite(
) {
    let root = support::workspace_root();
    let registry_raw = std::fs::read_to_string(root.join("configs/ci/registry/tool_registry.toml"))
        .expect("read configs/ci/registry/tool_registry.toml");
    let registry: toml::Value =
        registry_raw.parse().expect("parse configs/ci/registry/tool_registry.toml");

    let suite_files = std::fs::read_dir(root.join("crates/bijux-dna-bench/bench/suites"))
        .expect("read bench suite directory")
        .filter_map(|entry| entry.ok().map(|row| row.path()))
        .filter(|path| {
            path.file_name().and_then(|name| name.to_str()).is_some_and(|name| {
                std::path::Path::new(name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
            })
        })
        .collect::<Vec<_>>();

    let mut suite_tools = BTreeSet::new();
    for file in suite_files {
        let raw =
            std::fs::read_to_string(&file).unwrap_or_else(|_| panic!("read {}", file.display()));
        let parsed: toml::Value =
            raw.parse().unwrap_or_else(|_| panic!("parse {}", file.display()));
        if let Some(stages) = parsed.get("stages").and_then(toml::Value::as_array) {
            for stage in stages {
                for tool in list(stage, "tools") {
                    suite_tools.insert(tool);
                }
            }
        }
    }

    let tools = registry.get("tools").and_then(toml::Value::as_array).cloned().unwrap_or_default();
    let tool_by_id = tools
        .iter()
        .filter_map(|tool| {
            let id = tool.get("id").and_then(toml::Value::as_str)?;
            Some((id.to_string(), tool.clone()))
        })
        .collect::<BTreeMap<_, _>>();

    let benchmark_stages = [
        "fastq.validate_reads",
        "fastq.trim_reads",
        "fastq.filter_reads",
        "fastq.profile_reads",
        "fastq.report_qc",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();

    let stage_rows =
        registry.get("stages").and_then(toml::Value::as_array).cloned().unwrap_or_default();

    let mut required_tools = BTreeSet::new();
    for stage in stage_rows {
        let Some(stage_id) = stage.get("id").and_then(toml::Value::as_str) else {
            continue;
        };
        if !benchmark_stages.contains(stage_id) {
            continue;
        }
        let mut mapped = list(&stage, "primary_tools");
        mapped.extend(list(&stage, "optional_alternatives"));
        mapped.extend(list(&stage, "validation_tools"));
        mapped.extend(list(&stage, "reporting_tools"));
        mapped.sort();
        mapped.dedup();
        for tool_id in mapped {
            let Some(tool_row) = tool_by_id.get(&tool_id) else {
                continue;
            };
            let status =
                tool_row.get("status").and_then(toml::Value::as_str).unwrap_or("supported");
            if support::registry_status_is_production(status) {
                required_tools.insert(tool_id);
            }
        }
    }

    let mut offenders = Vec::new();
    for tool_id in &required_tools {
        if !suite_tools.contains(tool_id) {
            offenders.push(format!(
                "production benchmark tool {tool_id} missing from bench suite crates/bijux-dna-bench/bench/suites/*.toml"
            ));
        }
    }

    for tool_id in &required_tools {
        let Some(tool_row) = tool_by_id.get(tool_id) else {
            continue;
        };
        let version_cmd =
            tool_row.get("version_cmd").and_then(toml::Value::as_str).unwrap_or("").trim();
        let help_cmd = tool_row.get("help_cmd").and_then(toml::Value::as_str).unwrap_or("").trim();
        if version_cmd.is_empty() || help_cmd.is_empty() {
            offenders.push(format!(
                "production benchmark tool {tool_id} has smoke warning: missing version/help command"
            ));
        }
        if let Some(smoke_status) = tool_row.get("smoke_status").and_then(toml::Value::as_str) {
            if smoke_status.eq_ignore_ascii_case("warning")
                || smoke_status.eq_ignore_ascii_case("error")
            {
                offenders.push(format!(
                    "production benchmark tool {tool_id} has smoke_status={smoke_status} and cannot be production-ready"
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "benchmark suite support policy violations:\n{}",
        offenders.join("\n")
    );
}
