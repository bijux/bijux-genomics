#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;
use walkdir::WalkDir;

#[test]
fn policy__contracts__ci_no_stage_tool_defs_policy__workflows_must_not_define_stage_or_tool_ids() {
    let root = support::workspace_root();
    let registry_raw = std::fs::read_to_string(root.join("configs/ci/registry/tool_registry.toml"))
        .expect("read configs/ci/registry/tool_registry.toml");
    let parsed: toml::Value = registry_raw
        .parse()
        .expect("parse configs/ci/registry/tool_registry.toml");

    let mut ids = BTreeSet::new();
    if let Some(tools) = parsed.get("tools").and_then(toml::Value::as_array) {
        for tool in tools {
            if let Some(id) = tool.get("id").and_then(toml::Value::as_str) {
                ids.insert(id.to_string());
            }
        }
    }
    if let Some(stages) = parsed.get("stages").and_then(toml::Value::as_array) {
        for stage in stages {
            if let Some(id) = stage.get("id").and_then(toml::Value::as_str) {
                ids.insert(id.to_string());
            }
        }
    }

    let workflows = root.join(".github").join("workflows");
    let mut offenders = Vec::new();
    for entry in WalkDir::new(&workflows).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let Some(ext) = path.extension().and_then(std::ffi::OsStr::to_str) else {
            continue;
        };
        if ext != "yml" && ext != "yaml" {
            continue;
        }
        let content = std::fs::read_to_string(path).expect("read workflow");
        for id in &ids {
            if content.contains(id) {
                offenders.push(format!(
                    "{} contains SSOT id literal `{id}`",
                    path.display()
                ));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "CI workflows must not enumerate stage/tool IDs directly:\n{}",
        offenders.join("\n")
    );
}
