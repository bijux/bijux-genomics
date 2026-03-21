use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::{
    atomic_write_bytes, atomic_write_with, lock_run, pipeline_run_dir, retry_with,
    run_layout_paths, temp_dir, Clock, FileLock, IoError, IoErrorKind, RetryPolicy,
    PIPELINE_RUN_DIR_TEMPLATE, RUN_LAYOUT_CONTRACT,
};

struct FakeClock {
    sleeps: Arc<Mutex<Vec<Duration>>>,
}

impl Clock for FakeClock {
    fn sleep(&self, duration: Duration) {
        if let Ok(mut guard) = self.sleeps.lock() {
            guard.push(duration);
        }
    }
}

#[test]
fn retry_policy_is_deterministic() -> Result<(), IoError> {
    let sleeps = Arc::new(Mutex::new(Vec::new()));
    let clock = FakeClock {
        sleeps: sleeps.clone(),
    };
    let policy = RetryPolicy {
        max_attempts: 3,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(40),
    };
    let mut attempts = 0;
    let _ = retry_with(&policy, &clock, |_| {
        attempts += 1;
        Err::<(), _>(IoError::new(IoErrorKind::Transient, "fail"))
    });
    assert_eq!(attempts, 3);
    let recorded = sleeps
        .lock()
        .map_err(|_| IoError::new(IoErrorKind::Other, "lock poisoned"))?
        .clone();
    assert_eq!(
        recorded,
        vec![Duration::from_millis(10), Duration::from_millis(20)]
    );
    Ok(())
}

#[test]
fn atomic_write_failure_does_not_clobber() -> Result<(), IoError> {
    let dir = temp_dir("bijux")?;
    let target = dir.path().join("payload.json");
    atomic_write_bytes(&target, b"stable")?;
    let result = atomic_write_with(&target, |_| Err(std::io::Error::other("boom")));
    assert!(result.is_err());
    let data = std::fs::read_to_string(&target).map_err(IoError::from_io)?;
    assert_eq!(data, "stable");
    Ok(())
}

#[test]
fn lock_acquisition_times_out() -> Result<(), IoError> {
    let dir = temp_dir("bijux")?;
    let lock_path = dir.path().join("lockfile");
    let _lock = FileLock::acquire(&lock_path, Duration::from_millis(50))?;
    let err = FileLock::acquire(&lock_path, Duration::from_millis(50))
        .err()
        .ok_or_else(|| IoError::new(IoErrorKind::Other, "expected lock timeout"))?;
    assert_eq!(err.kind, IoErrorKind::LockTimeout);
    Ok(())
}

#[test]
fn run_layout_is_stable() {
    let base = Path::new("/tmp/bijux");
    let layout = run_layout_paths(base, "run-123");
    assert_eq!(layout.run_dir, base.join("runs").join("run-123"));
    assert_eq!(
        layout.artifacts_dir,
        base.join("runs").join("run-123").join("artifacts")
    );
    assert_eq!(
        layout.logs_dir,
        base.join("runs").join("run-123").join("logs")
    );
    assert_eq!(
        layout.tmp_dir,
        base.join("runs").join("run-123").join("tmp")
    );
}

#[test]
fn run_layout_contract_is_enforced() -> Result<(), IoError> {
    let dir = temp_dir("bijux")?;
    let layout = run_layout_paths(dir.path(), "run-1");
    let _lock = lock_run(&layout, Duration::from_millis(50))?;
    let marker = crate::publish_run(&layout, "run-1")?;
    assert!(marker.ends_with(RUN_LAYOUT_CONTRACT.publish_marker));
    Ok(())
}

#[test]
fn pipeline_run_dir_contract_is_stable() {
    let base = Path::new("/tmp/bijux");
    let path = pipeline_run_dir(base, "fastq-to-fastq__default__v1", "sample-1", "run-abc");
    assert_eq!(
        path,
        PathBuf::from("/tmp/bijux")
            .join("fastq-to-fastq__default__v1")
            .join("sample-1")
            .join("run-abc")
    );
    assert_eq!(
        PIPELINE_RUN_DIR_TEMPLATE,
        "{pipeline_id}/{sample_id}/{run_id}"
    );
}

#[test]
fn temp_dir_is_created() -> Result<(), IoError> {
    let dir = temp_dir("bijux-dna-test")?;
    assert!(dir.path().exists());
    Ok(())
}

#[test]
fn bounded_read_rejects_files_larger_than_limit() -> Result<(), IoError> {
    let dir = temp_dir("bijux")?;
    let path = dir.path().join("payload.bin");
    atomic_write_bytes(&path, b"abcdef")?;

    let exact = crate::read_to_end_bounded(&path, 6)?;
    assert_eq!(exact, b"abcdef");

    let err = crate::read_to_end_bounded(&path, 5)
        .err()
        .ok_or_else(|| IoError::new(IoErrorKind::Other, "expected bounded read failure"))?;
    assert_eq!(err.kind, IoErrorKind::Corruption);
    Ok(())
}
