use std::fs;
use std::path::PathBuf;

#[test]
fn fastq_stage_docs_distinguish_admitted_and_experimental_correctors() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let stage_list = fs::read_to_string(root.join("docs").join("STAGE_LIST.md"))
        .expect("read STAGE_LIST.md");
    let planner_mapping = fs::read_to_string(
        root.parent()
            .and_then(std::path::Path::parent)
            .expect("workspace root")
            .join("crates")
            .join("bijux-dna-planner-fastq")
            .join("docs")
            .join("STAGE_MAPPING.md"),
    )
    .expect("read planner STAGE_MAPPING.md");

    for content in [&stage_list, &planner_mapping] {
        let row = content
            .lines()
            .find(|line| line.trim_start().starts_with("| fastq.correct_errors "))
            .expect("fastq.correct_errors row");
        assert!(
            row.contains("rcorrector"),
            "correct-errors docs must name the admitted default corrector"
        );
        assert!(
            row.contains("experimental"),
            "correct-errors docs must distinguish experimental backends"
        );
    }
}
