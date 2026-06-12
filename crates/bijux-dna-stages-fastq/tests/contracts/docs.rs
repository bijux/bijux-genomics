#![allow(clippy::expect_used)]

use std::fs;
use std::path::PathBuf;

#[test]
fn fastq_stage_docs_describe_governed_corrector_surface_and_observer_coverage() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let stage_contracts = fs::read_to_string(root.join("docs").join("STAGE_CONTRACTS.md"))
        .expect("read STAGE_CONTRACTS.md");

    let row = stage_contracts
        .lines()
        .find(|line| line.trim_start().starts_with("| `fastq.correct_errors` "))
        .expect("fastq.correct_errors row");
    assert!(
        row.contains("single-end or paired FASTQ"),
        "correct-errors docs must describe the governed input surface"
    );
    assert!(
        row.contains("governed correction report"),
        "correct-errors docs must describe the governed report contract"
    );
    assert!(
        stage_contracts.contains("domain/fastq/execution_support.yaml"),
        "stage contract docs must point readers to execution support truth"
    );

    assert!(
        stage_contracts.contains("observer_stage_ids()"),
        "stage contracts docs must distinguish contract coverage from observer coverage",
    );
    assert!(
        stage_contracts.contains("fastq.detect_adapters"),
        "observer docs must list the observer-specialized FASTQ stages",
    );
}
