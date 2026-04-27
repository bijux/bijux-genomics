#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn parse_tool_ids(tool_registry_path: &Path) -> Vec<String> {
    let raw = std::fs::read_to_string(tool_registry_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", tool_registry_path.display()));
    let parsed = raw
        .parse::<toml::Value>()
        .unwrap_or_else(|err| panic!("parse {}: {err}", tool_registry_path.display()));
    let mut ids = parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| {
            entry.get("id").and_then(toml::Value::as_str).map(std::string::ToString::to_string)
        })
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    ids
}

#[test]
fn policy__contracts__make_script_enumeration_policy__tool_stage_lists_live_in_registry_only() {
    let root = repo_root();
    let tool_ids = parse_tool_ids(
        &root.join("configs").join("ci").join("registry").join("tool_registry.toml"),
    );
    let stage_markers = ["fastq.", "bam.", "vcf."];
    let mut offenders = Vec::new();

    for rel in ["makes", "scripts"] {
        for entry in WalkDir::new(root.join(rel))
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let path = entry.path();
            let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
                continue;
            };
            if !matches!(ext, "mk" | "sh") {
                continue;
            }
            let content = std::fs::read_to_string(path)
                .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
            let tool_hits = tool_ids.iter().filter(|tool| content.contains(*tool)).count();
            let stage_hits =
                stage_markers.iter().filter(|marker| content.contains(*marker)).count();
            if tool_hits >= 3 || stage_hits >= 3 {
                offenders.push(path.display().to_string());
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "tool/stage enumeration must stay in registry/domain SSOT surfaces:\n{}",
        offenders.join("\n")
    );
}
