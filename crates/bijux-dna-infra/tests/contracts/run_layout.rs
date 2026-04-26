use std::path::{Path, PathBuf};
use std::time::Duration;

#[test]
fn lock_acquisition_times_out() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let lock_path = dir.path().join("lockfile");
    let _lock = bijux_dna_infra::FileLock::acquire(&lock_path, Duration::from_millis(50))?;
    let err = bijux_dna_infra::FileLock::acquire(&lock_path, Duration::from_millis(50))
        .err()
        .ok_or_else(|| anyhow::anyhow!("expected lock timeout"))?;
    assert_eq!(err.kind, bijux_dna_infra::IoErrorKind::LockTimeout);
    Ok(())
}

#[test]
fn file_lock_creates_parent_directory() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let lock_path = dir.path().join("nested").join("lockfile");

    let _lock = bijux_dna_infra::FileLock::acquire(&lock_path, Duration::from_millis(50))?;

    assert!(lock_path.exists());
    Ok(())
}

#[test]
fn run_layout_is_stable() {
    let base = Path::new("/tmp/bijux");
    let layout = bijux_dna_infra::run_layout_paths(base, "run-123");
    assert_eq!(layout.run_dir, base.join("runs").join("run-123"));
    assert_eq!(layout.artifacts_dir, base.join("runs").join("run-123").join("artifacts"));
    assert_eq!(layout.logs_dir, base.join("runs").join("run-123").join("logs"));
    assert_eq!(layout.tmp_dir, base.join("runs").join("run-123").join("tmp"));
}

#[test]
fn run_layout_paths_confine_run_id() {
    let base = Path::new("/tmp/bijux");
    let layout = bijux_dna_infra::run_layout_paths(base, "../run/123");
    assert_eq!(layout.run_dir, base.join("runs").join("run_123"));
    assert_eq!(layout.artifacts_dir, base.join("runs").join("run_123").join("artifacts"));
}

#[test]
fn run_layout_contract_is_enforced() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let layout = bijux_dna_infra::run_layout_paths(dir.path(), "run-1");
    let _lock = bijux_dna_infra::lock_run(&layout, Duration::from_millis(50))?;
    let marker = bijux_dna_infra::publish_run(&layout, "run-1")?;
    assert!(marker.ends_with(bijux_dna_infra::RUN_LAYOUT_CONTRACT.publish_marker));
    Ok(())
}

#[test]
fn pipeline_run_dir_contract_is_stable() {
    let base = Path::new("/tmp/bijux");
    let path = bijux_dna_infra::pipeline_run_dir(
        base,
        "fastq-to-fastq__default__v1",
        "sample-1",
        "run-abc",
    );
    assert_eq!(
        path,
        PathBuf::from("/tmp/bijux")
            .join("fastq-to-fastq__default__v1")
            .join("sample-1")
            .join("run-abc")
    );
    assert_eq!(bijux_dna_infra::PIPELINE_RUN_DIR_TEMPLATE, "{pipeline_id}/{sample_id}/{run_id}");
}

#[test]
fn pipeline_run_dir_confines_dynamic_segments() {
    let base = Path::new("/tmp/bijux");
    let path = bijux_dna_infra::pipeline_run_dir(base, "/absolute/pipeline", "../sample", "run/a");
    assert_eq!(
        path,
        PathBuf::from("/tmp/bijux").join("absolute_pipeline").join("sample").join("run_a")
    );
}
