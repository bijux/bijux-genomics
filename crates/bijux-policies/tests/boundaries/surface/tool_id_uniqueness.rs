#![allow(non_snake_case)]
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn extract_tool_ids(path: &Path) -> Vec<String> {
    let content = std::fs::read_to_string(path).expect("read tool registry");
    let mut ids = Vec::new();
    let re = regex::Regex::new(r#"\(\s*\"([^\"]+)\"\s*,\s*\"([^\"]+)\"\s*\)"#)
        .expect("regex");
    for cap in re.captures_iter(&content) {
        ids.push(cap[1].to_string());
    }
    ids
}

#[test]
fn policy__surface__tool_id_uniqueness__tool_ids_are_unique_across_planners() {
    let root = workspace_root();
    let registries = [
        root.join("crates/bijux-planner-fastq/src/selection/tool_registry.rs"),
        root.join("crates/bijux-planner-bam/src/selection/tool_registry.rs"),
    ];
    let allowlist = ["samtools"]; // shared tool identity across pipelines

    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    let mut offenders = Vec::new();
    for registry in registries {
        for tool_id in extract_tool_ids(&registry) {
            if allowlist.contains(&tool_id.as_str()) {
                continue;
            }
            if let Some(existing) = seen.get(&tool_id) {
                offenders.push(format!(
                    "tool id `{}` appears in both {} and {}",
                    tool_id,
                    existing,
                    registry.display()
                ));
            } else {
                seen.insert(tool_id, registry.display().to_string());
            }
        }
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "Tool IDs must be globally unique across planners.\n\
If a tool is intentionally shared, add it to the allowlist with justification.\n\
See docs/40-policies/STYLE.md for tool identity rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
