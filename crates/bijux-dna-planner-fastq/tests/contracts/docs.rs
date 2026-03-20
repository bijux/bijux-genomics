use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use bijux_dna_core::ids::StageId;
use toml::Value;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn read_doc(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn parse_toml(path: &Path) -> Value {
    let raw = read_doc(path);
    raw.parse::<Value>()
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

fn stage_mapping_rows() -> Vec<(String, String)> {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("STAGE_MAPPING.md");
    let content = read_doc(&doc);
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("| fastq.") {
                return None;
            }
            let cells = trimmed
                .trim_start_matches('|')
                .trim_end_matches('|')
                .split('|')
                .map(str::trim)
                .collect::<Vec<_>>();
            if cells.len() < 2 {
                return None;
            }
            Some((cells[0].to_string(), cells[1].to_string()))
        })
        .collect()
}

#[test]
fn stage_mapping_covers_planner_registry() {
    let documented = stage_mapping_rows()
        .into_iter()
        .map(|(stage_id, _)| stage_id)
        .collect::<BTreeSet<_>>();
    let registry = bijux_dna_planner_fastq::stage_api::fastq::registry()
        .into_iter()
        .map(|stage| stage.id().to_string())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        documented, registry,
        "STAGE_MAPPING.md drifted from planner registry"
    );
}

#[test]
fn stage_mapping_tool_lists_match_planner_admission() {
    for (stage_id_raw, tool_cell) in stage_mapping_rows() {
        let stage_id = StageId::new(&stage_id_raw);
        let documented = tool_cell
            .split(',')
            .map(str::trim)
            .filter(|tool| !tool.is_empty())
            .map(str::to_string)
            .collect::<BTreeSet<_>>();
        let admitted = bijux_dna_planner_fastq::stage_api::allowed_tools_for_stage(&stage_id)
            .into_iter()
            .map(|tool| tool.to_string())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            documented, admitted,
            "STAGE_MAPPING.md tool list drifted for {stage_id_raw}"
        );
    }
}

#[test]
fn tool_selection_doc_describes_closed_runtime_boundary() {
    let doc = read_doc(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("docs")
            .join("TOOL_SELECTION.md"),
    );
    assert!(
        doc.contains("publish only the\nclosed FASTQ execution surface")
            || doc.contains("publish only the closed FASTQ execution surface"),
        "TOOL_SELECTION.md must explain that generated configs expose only closed runtime support",
    );
}

#[test]
fn ci_stage_catalog_excludes_declared_only_fastq_stages() {
    let stages = parse_toml(&workspace_root().join("configs/ci/stages/stages.toml"));
    let stage_ids = stages
        .get("stages")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|stage| stage.get("id").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();
    assert!(
        !stage_ids.contains("fastq.infer_asvs"),
        "declared-only FASTQ stages must stay out of the governed CI stage catalog",
    );
}

#[test]
fn ci_tool_registry_excludes_planned_fastq_tools() {
    let registry = parse_toml(&workspace_root().join("configs/ci/registry/tool_registry.toml"));
    let tool_ids = registry
        .get("tools")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|tool| tool.get("id").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();
    for tool_id in [
        "dada2",
        "diamond",
        "dustmasker",
        "fastq_scan",
        "seqfu",
        "seqpurge",
    ] {
        assert!(
            !tool_ids.contains(tool_id),
            "planned FASTQ tool {tool_id} must stay out of the governed CI runtime registry",
        );
    }
}
