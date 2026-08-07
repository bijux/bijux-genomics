use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_containers::api::{
    load_run_evidence_index, run_evidence_index_schema, validate_run_evidence_index,
    AttemptExecutionStatus,
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
    assert_eq!(index.summary.run_count, index.runs.len());
    assert_eq!(index.summary.tool_count, 71);
    assert!(run_evidence_index_schema().as_value().is_object());
    Ok(())
}

#[test]
fn selected_run_summary_drift_is_rejected() -> Result<()> {
    let mut index =
        load_run_evidence_index(&repository_root().join("containers/reports/run-evidence.json"))?;
    index.summary.run_count += 1;

    assert!(validate_run_evidence_index(&index)
        .is_err_and(|error| error.to_string().contains("summary run count drifted")));
    Ok(())
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("container crate must live under repository crates/"))
        .to_path_buf()
}
