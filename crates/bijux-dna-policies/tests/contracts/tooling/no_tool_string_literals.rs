#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;
use walkdir::WalkDir;

fn registry_tool_ids() -> BTreeSet<String> {
    let registry_path = support::workspace_root().join("configs/tools.toml");
    let raw = std::fs::read_to_string(&registry_path).expect("read configs/tools.toml");
    let parsed: toml::Value = raw.parse().expect("parse configs/tools.toml");
    parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|tool| tool.get("id").and_then(toml::Value::as_str))
                .map(str::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default()
}

#[test]
fn policy__contracts__tooling__no_tool_string_literals__stages_and_planners_use_registry_toolid() {
    let root = support::workspace_root();
    let tool_ids = registry_tool_ids();
    let scan_roots = [
        root.join("crates/bijux-dna-stages-fastq/src"),
        root.join("crates/bijux-dna-stages-bam/src"),
        root.join("crates/bijux-dna-planner-fastq/src"),
        root.join("crates/bijux-dna-planner-bam/src"),
    ];
    let allow_suffixes = [
        "selection/tool_registry.rs",
        "selection/id_literals.rs",
        "selection/catalog.rs",
    ];

    let mut offenders = Vec::new();
    for scan_root in scan_roots {
        for entry in WalkDir::new(scan_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            let rel = path
                .strip_prefix(&root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
            if allow_suffixes.iter().any(|suffix| rel.ends_with(suffix)) {
                continue;
            }
            let content = std::fs::read_to_string(path).expect("read source");
            for id in &tool_ids {
                let needle = format!("\"{id}\"");
                if content.contains(&needle) {
                    offenders.push(format!(
                        "{}: contains raw tool id literal {needle}; use ToolId from registry",
                        rel
                    ));
                }
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "raw tool id literal policy violations:\n{}",
        offenders.join("\n")
    );
}
