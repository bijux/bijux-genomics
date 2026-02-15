#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

#[test]
fn policy__contracts__planner_data_driven_catalog_policy__core_catalogs_do_not_use_stage_all() {
    let root = support::workspace_root();
    let targets = [
        root.join("crates/bijux-dna-planner-fastq/src"),
        root.join("crates/bijux-dna-planner-bam/src"),
        root.join("crates/bijux-dna-pipelines/src"),
    ];

    let mut offenders = Vec::new();
    for target in targets {
        for entry in WalkDir::new(&target).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if !entry.file_type().is_file() {
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            if path.to_string_lossy().contains("/tests/") {
                continue;
            }
            let content = std::fs::read_to_string(path).expect("read source");
            if content.contains("BamStage::all(") {
                offenders.push(path.display().to_string());
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "stage catalogs must be data-driven from generated registry, not BamStage::all():\n{}",
        offenders.join("\n")
    );
}
