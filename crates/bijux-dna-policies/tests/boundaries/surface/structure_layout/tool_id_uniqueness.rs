#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::BTreeSet;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn registry_path() -> PathBuf {
    repo_root().join("configs/ci/registry/tool_registry.toml")
}

#[test]
fn policy__boundaries__tool_id_uniqueness__tool_ids_are_unique_across_planners() {
    let path = registry_path();
    let content = std::fs::read_to_string(&path).expect("read tool registry");
    let parsed: toml::Value = content.parse().expect("parse tool registry");
    let tools = parsed.get("tools").and_then(toml::Value::as_array).cloned().unwrap_or_default();
    let mut seen = BTreeSet::new();
    let mut offenders = Vec::new();
    for entry in tools {
        let Some(id) = entry.get("id").and_then(toml::Value::as_str) else {
            continue;
        };
        if !seen.insert(id.to_string()) {
            offenders.push(format!("duplicate tool id `{id}` in {}", path.display()));
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Tool IDs must be globally unique in generated registry.\n\
See docs/40-policies/STYLE.md for tool identity rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
