#![allow(non_snake_case)]
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__boundaries__planner_purity__planners_do_not_define_parsers() {
    let root = repo_root();
    let planners = [
        root.join("crates/bijux-dna-planner-fastq/src"),
        root.join("crates/bijux-dna-planner-bam/src"),
    ];
    let mut offenders = Vec::new();
    for planner in planners {
        for entry in walkdir::WalkDir::new(&planner).min_depth(1).max_depth(6) {
            let entry = entry.expect("walk entry");
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let content = std::fs::read_to_string(entry.path()).expect("read source");
            if content.contains("fn parse_") {
                offenders.push(entry.path().display().to_string());
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Planners must not define parsing logic (parsers live in stages).\n\
Move parser functions into bijux-dna-stages-* crates.\n\
See docs/40-policies/STYLE.md for planner purity rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
