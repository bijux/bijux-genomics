use super::assess_execution;
use std::path::PathBuf;

#[test]
fn assess_execution_success() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let output = dir.path().join("out.data");
    bijux_dna_infra::atomic_write_bytes(&output, b"ok")?;
    let assessment = assess_execution(0, &[output]);
    assert!(assessment.success);
    Ok(())
}

#[test]
fn assess_execution_missing_outputs() {
    let missing = PathBuf::from("/artifacts/runtime/missing.data");
    let assessment = assess_execution(0, &[missing]);
    assert!(!assessment.success);
    assert_eq!(assessment.reason.as_deref(), Some("missing_outputs"));
}

#[test]
fn assess_execution_partial_outputs() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let present = dir.path().join("present.data");
    bijux_dna_infra::atomic_write_bytes(&present, b"ok")?;
    let missing = dir.path().join("missing.data");
    let assessment = assess_execution(0, &[present, missing]);
    assert!(!assessment.success);
    assert_eq!(assessment.reason.as_deref(), Some("missing_outputs"));
    Ok(())
}

#[test]
fn assess_execution_bad_exit_code() {
    let assessment = assess_execution(1, &[]);
    assert!(!assessment.success);
    assert_eq!(assessment.reason.as_deref(), Some("exit_code=1"));
}
