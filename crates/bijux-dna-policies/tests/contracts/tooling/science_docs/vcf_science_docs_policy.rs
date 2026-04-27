#![allow(dead_code, non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq)]
struct VcfStageDocSpec {
    status: String,
    compatible_tools: BTreeSet<String>,
}

fn markdown_link_targets(path: &str) -> BTreeSet<String> {
    let root = support::workspace_root();
    let raw =
        fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"));
    let mut targets = BTreeSet::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("- [") {
            continue;
        }
        if let Some((_, rest)) = trimmed.split_once("](") {
            if let Some((target, _)) = rest.split_once(')') {
                targets.insert(target.to_string());
            }
        }
    }
    targets
}

fn markdown_table_rows(path: &str, header_prefix: &str) -> Vec<Vec<String>> {
    let root = support::workspace_root();
    let raw =
        fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"));
    let mut rows = Vec::new();
    let mut in_table = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(header_prefix) {
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }
        if trimmed.starts_with("|---") || trimmed.starts_with("| ---") {
            continue;
        }
        if trimmed.starts_with('#') || trimmed.is_empty() {
            break;
        }
        if trimmed.starts_with('|') {
            rows.push(
                trimmed
                    .trim_matches('|')
                    .split('|')
                    .map(|value| value.trim().to_string())
                    .collect(),
            );
        }
    }
    rows
}

fn vcf_science_doc_targets() -> BTreeSet<String> {
    let root = support::workspace_root();
    fs::read_dir(root.join("docs/20-science/vcf"))
        .expect("read docs/20-science/vcf")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
        .filter(|path| path.file_name().and_then(|name| name.to_str()) != Some("index.md"))
        .map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_else(|| panic!("invalid UTF-8 doc path {}", path.display()))
                .to_string()
        })
        .collect()
}

fn vcf_stage_specs() -> BTreeMap<String, VcfStageDocSpec> {
    let root = support::workspace_root();
    let mut specs = BTreeMap::new();
    for entry in fs::read_dir(root.join("domain/vcf/stages")).expect("read vcf stages") {
        let path = entry.expect("stage entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read VCF stage manifest {}: {err}", path.display()));
        let stage_id = raw
            .lines()
            .find_map(|line| line.strip_prefix("stage_id: "))
            .map(|value| value.trim_matches('"').to_string())
            .unwrap_or_else(|| panic!("missing stage_id in {}", path.display()));
        let status = raw
            .lines()
            .find_map(|line| line.strip_prefix("status: "))
            .map(|value| value.trim_matches('"').to_string())
            .unwrap_or_else(|| panic!("missing status in {}", path.display()));
        let mut compatible_tools = BTreeSet::new();
        let mut in_tools = false;
        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed == "compatible_tools:" {
                in_tools = true;
                continue;
            }
            if in_tools {
                if let Some(tool_id) = trimmed.strip_prefix("- ") {
                    compatible_tools.insert(tool_id.trim_matches('"').to_string());
                    continue;
                }
                if let Some(value) = trimmed.strip_prefix("compatible_tools: [") {
                    for tool_id in value.trim_end_matches(']').split(',') {
                        let tool_id = tool_id.trim().trim_matches('"');
                        if !tool_id.is_empty() {
                            compatible_tools.insert(tool_id.to_string());
                        }
                    }
                    in_tools = false;
                    continue;
                }
                in_tools = false;
            }
        }
        specs.insert(stage_id, VcfStageDocSpec { status, compatible_tools });
    }
    specs
}

#[test]
fn policy__contracts__vcf_science_docs_policy__index_covers_vcf_science_docs_exactly() {
    let expected = vcf_science_doc_targets();
    let documented = markdown_link_targets("docs/20-science/vcf/index.md");
    assert_eq!(
        expected, documented,
        "VCF science index must link every VCF science doc exactly once"
    );
}
