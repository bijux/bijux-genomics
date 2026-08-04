use anyhow::{Context, Result};

fn checked_in_run_evidence_path() -> Result<std::path::PathBuf> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .context("runtime crate is outside the repository workspace")?;
    Ok(root.join("containers/reports/run-evidence.json"))
}

#[test]
fn checked_in_run_evidence_is_admitted_without_rewriting_history() -> Result<()> {
    let path = checked_in_run_evidence_path()?;
    let before = std::fs::read(&path)?;
    let index = bijux_dna_runtime::load_run_evidence_index(&path)?;
    let after = std::fs::read(path)?;

    assert_eq!(index.runs.len(), 71);
    assert_eq!(before, after);
    assert!(index.runs.iter().all(|record| {
        record.execution_status == bijux_dna_runtime::AttemptExecutionStatus::Passed
    }));
    Ok(())
}

#[test]
fn run_evidence_rejects_duplicate_run_ownership() -> Result<()> {
    let mut index = bijux_dna_runtime::load_run_evidence_index(&checked_in_run_evidence_path()?)?;
    let duplicate =
        index.runs.first().cloned().context("checked-in run-evidence index is empty")?;
    index.runs.push(duplicate);
    let error =
        bijux_dna_runtime::validate_run_evidence_index(&index).err().map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("duplicates run")));
    Ok(())
}
