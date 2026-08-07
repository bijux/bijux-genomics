use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_containers::api::{
    load_run_evidence_index, validate_run_evidence_index, AttemptExecutionStatus,
};

#[test]
fn checked_in_selected_runs_are_admitted_by_the_container_owner() -> Result<()> {
    let index =
        load_run_evidence_index(&repository_root().join("containers/reports/run-evidence.json"))?;

    validate_run_evidence_index(&index)?;
    assert!(!index.runs.is_empty());
    assert!(index
        .runs
        .iter()
        .all(|record| record.execution_status == AttemptExecutionStatus::Passed));
    Ok(())
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("container crate must live under repository crates/"))
        .to_path_buf()
}
