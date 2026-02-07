use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn planners_do_not_define_parsers() {
    let root = workspace_root();
    let planners = [
        root.join("crates/bijux-planner-fastq/src"),
        root.join("crates/bijux-planner-bam/src"),
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
    assert!(
        offenders.is_empty(),
        "Planners must not define parsing logic (parsers live in stages).\n\
Move parser functions into bijux-stages-* crates.\n\
See docs/STYLE.md for planner purity rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
