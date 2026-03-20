use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use bijux_dna_core::ids::StageId;

fn stage_mapping_rows() -> Vec<(String, String)> {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("STAGE_MAPPING.md");
    let content = fs::read_to_string(&doc).expect("read STAGE_MAPPING.md");
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
