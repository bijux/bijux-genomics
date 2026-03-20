use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

#[test]
fn stage_mapping_covers_planner_registry() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("STAGE_MAPPING.md");
    let content = fs::read_to_string(&doc).expect("read STAGE_MAPPING.md");
    let documented = content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("| fastq.") {
                return None;
            }
            trimmed
                .trim_start_matches('|')
                .split('|')
                .next()
                .map(str::trim)
                .map(str::to_string)
        })
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
fn stage_mapping_screen_taxonomy_matches_admitted_tools() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("STAGE_MAPPING.md");
    let content = fs::read_to_string(&doc).expect("read STAGE_MAPPING.md");
    let row = content
        .lines()
        .find(|line| line.trim_start().starts_with("| fastq.screen_taxonomy "))
        .expect("fastq.screen_taxonomy row");

    assert!(
        !row.contains("metaphlan"),
        "screen taxonomy docs must not advertise metaphlan"
    );
    assert!(
        !row.contains("fastq_screen"),
        "screen taxonomy docs must not advertise fastq_screen"
    );
}
