use std::fs;
use std::path::PathBuf;

#[test]
fn fastq_stage_docs_describe_closed_corrector_and_observer_coverage() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let stage_list = fs::read_to_string(root.join("docs").join("STAGE_LIST.md"))
        .expect("read STAGE_LIST.md");
    let stage_contracts = fs::read_to_string(root.join("docs").join("STAGE_CONTRACTS.md"))
        .expect("read STAGE_CONTRACTS.md");
    let observers = fs::read_to_string(root.join("docs").join("OBSERVERS.md"))
        .expect("read OBSERVERS.md");
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
            "correct-errors docs must name the closed execution backend"
        );
        assert!(
            !row.contains("experimental"),
            "correct-errors docs must not advertise environment-driven experimental backends"
        );
    }

    assert!(
        stage_contracts.contains("observer_stage_ids()"),
        "stage contracts docs must distinguish contract coverage from observer coverage",
    );
    assert!(
        observers.contains("fastq.detect_adapters"),
        "observer docs must list the observer-specialized FASTQ stages",
    );
}
