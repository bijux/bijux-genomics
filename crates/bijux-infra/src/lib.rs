use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use thiserror::Error;

mod logging;
mod paths;

pub use logging::init_logging;
pub use paths::{bench_base_dir, bench_tools_dir};

pub mod formats;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoErrorKind {
    Permission,
    Missing,
    Transient,
    Corruption,
    LockTimeout,
    Other,
}

#[derive(Debug, Error)]
#[error("{kind:?}: {message}")]
pub struct IoError {
    pub kind: IoErrorKind,
    pub message: String,
    #[source]
    pub source: Option<std::io::Error>,
}

impl IoError {
    #[must_use]
    pub fn from_io(err: std::io::Error) -> Self {
        let kind = classify_io_error(&err);
        Self {
            kind,
            message: err.to_string(),
            source: Some(err),
        }
    }

    #[must_use]
    pub fn new(kind: IoErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            source: None,
        }
    }
}

#[must_use]
pub fn classify_io_error(err: &std::io::Error) -> IoErrorKind {
    use std::io::ErrorKind;
    match err.kind() {
        ErrorKind::NotFound => IoErrorKind::Missing,
        ErrorKind::PermissionDenied => IoErrorKind::Permission,
        ErrorKind::TimedOut | ErrorKind::WouldBlock | ErrorKind::Interrupted => {
            IoErrorKind::Transient
        }
        ErrorKind::InvalidData | ErrorKind::InvalidInput | ErrorKind::UnexpectedEof => {
            IoErrorKind::Corruption
        }
        _ => IoErrorKind::Other,
    }
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 4,
            base_delay: Duration::from_millis(50),
            max_delay: Duration::from_millis(500),
        }
    }
}

pub trait Clock {
    fn sleep(&self, duration: Duration);
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

/// Retry an operation with exponential backoff.
///
/// # Errors
/// Returns the last error from the operation after exhausting retries.
pub fn retry_with<T, E, F, C>(policy: &RetryPolicy, clock: &C, mut op: F) -> Result<T, E>
where
    F: FnMut(u32) -> Result<T, E>,
    C: Clock,
{
    let mut attempt = 0;
    loop {
        attempt += 1;
        match op(attempt) {
            Ok(value) => return Ok(value),
            Err(_err) if attempt < policy.max_attempts => {
                let delay = backoff_delay(policy, attempt);
                clock.sleep(delay);
            }
            Err(err) => return Err(err),
        }
    }
}

#[must_use]
pub fn backoff_delay(policy: &RetryPolicy, attempt: u32) -> Duration {
    let shift = attempt.saturating_sub(1).min(31);
    let pow = 1u32 << shift;
    let delay = policy.base_delay.saturating_mul(pow);
    delay.min(policy.max_delay)
}

/// Ensure a directory exists, creating it if needed.
///
/// # Errors
/// Returns an IO error if the directory cannot be created.
pub fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<(), IoError> {
    std::fs::create_dir_all(path.as_ref()).map_err(IoError::from_io)
}

/// Atomically write bytes to a path (temp + rename).
///
/// # Errors
/// Returns an IO error if the write or rename fails.
pub fn atomic_write_bytes(path: &Path, bytes: &[u8]) -> Result<(), IoError> {
    atomic_write_with(path, |file| file.write_all(bytes))
}

/// Write bytes to a path with the standard atomic write policy.
///
/// # Errors
/// Returns an IO error if serialization or writing fails.
pub fn write_bytes<P: AsRef<Path>, B: AsRef<[u8]>>(path: P, bytes: B) -> Result<(), IoError> {
    atomic_write_bytes(path.as_ref(), bytes.as_ref())
}

/// Write a UTF-8 string to a path with the standard atomic write policy.
///
/// # Errors
/// Returns an IO error if writing fails.
pub fn write_string<P: AsRef<Path>>(path: P, contents: &str) -> Result<(), IoError> {
    write_bytes(path, contents.as_bytes())
}

/// Atomically write JSON to a path (temp + rename).
///
/// # Errors
/// Returns an IO error if serialization or writing fails.
pub fn atomic_write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), IoError> {
    let raw = serde_json::to_value(value)
        .map_err(|err| IoError::new(IoErrorKind::Corruption, format!("serialize json: {err}")))?;
    let canonical = canonicalize_json_value(&raw);
    let payload = serde_json::to_vec_pretty(&canonical)
        .map_err(|err| IoError::new(IoErrorKind::Corruption, format!("serialize json: {err}")))?;
    atomic_write_bytes(path, &payload)
}

#[must_use]
pub fn canonicalize_json_value(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut ordered = serde_json::Map::new();
            for key in keys {
                let val = map.get(key).unwrap_or(&serde_json::Value::Null);
                ordered.insert(key.clone(), canonicalize_json_value(val));
            }
            serde_json::Value::Object(ordered)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(canonicalize_json_value).collect())
        }
        _ => value.clone(),
    }
}

/// Atomically write using a custom writer function.
///
/// # Errors
/// Returns an IO error if the write or rename fails.
pub fn atomic_write_with<F>(path: &Path, writer: F) -> Result<(), IoError>
where
    F: FnOnce(&mut File) -> std::io::Result<()>,
{
    let parent = path
        .parent()
        .ok_or_else(|| IoError::new(IoErrorKind::Missing, "path has no parent"))?;
    ensure_dir(parent)?;

    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(IoError::from_io)?;
    writer(temp.as_file_mut()).map_err(IoError::from_io)?;
    temp.as_file_mut().sync_all().map_err(IoError::from_io)?;
    let perm = default_permissions();
    if let Some(perm) = perm {
        temp.as_file_mut()
            .set_permissions(perm)
            .map_err(IoError::from_io)?;
    }
    temp.persist(path)
        .map_err(|err| IoError::from_io(err.error))?;
    Ok(())
}

/// Atomically write bytes with retry/backoff.
///
/// # Errors
/// Returns the last IO error after exhausting retries.
pub fn atomic_write_bytes_with_retry(
    path: &Path,
    bytes: &[u8],
    policy: &RetryPolicy,
) -> Result<(), IoError> {
    retry_with(policy, &SystemClock, |_| atomic_write_bytes(path, bytes))
}

/// Read a file with a maximum byte limit.
///
/// # Errors
/// Returns an IO error if reading fails or the file exceeds the limit.
pub fn read_to_end_bounded(path: &Path, max_bytes: usize) -> Result<Vec<u8>, IoError> {
    let mut file = File::open(path).map_err(IoError::from_io)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(IoError::from_io)?;
    if buffer.len() > max_bytes {
        return Err(IoError::new(
            IoErrorKind::Corruption,
            format!("file exceeds max bytes ({max_bytes})"),
        ));
    }
    Ok(buffer)
}

/// Rename a filesystem path.
///
/// # Errors
/// Returns an IO error if the rename fails.
pub fn rename(src: &Path, dst: &Path) -> Result<(), IoError> {
    std::fs::rename(src, dst).map_err(IoError::from_io)
}

/// Remove a file.
///
/// # Errors
/// Returns an IO error if the removal fails.
pub fn remove_file(path: &Path) -> Result<(), IoError> {
    std::fs::remove_file(path).map_err(IoError::from_io)
}

/// Remove a directory and all contents.
///
/// # Errors
/// Returns an IO error if removal fails.
pub fn remove_dir_all(path: &Path) -> Result<(), IoError> {
    std::fs::remove_dir_all(path).map_err(IoError::from_io)
}

/// Remove a file or directory if it exists.
///
/// # Errors
/// Returns an IO error if removal fails.
pub fn remove_path_if_exists(path: &Path) -> Result<(), IoError> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        remove_dir_all(path)
    } else {
        remove_file(path)
    }
}

/// Remove a file if it exists.
///
/// # Errors
/// Returns an IO error for failures other than missing files.
pub fn remove_file_if_exists(path: &Path) -> Result<(), IoError> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(IoError::from_io(err)),
    }
}

#[derive(Debug)]
pub struct FileLock {
    file: File,
}

impl FileLock {
    /// Acquire an exclusive lock on a file within a timeout.
    ///
    /// # Errors
    /// Returns a lock timeout error or IO error on failure.
    pub fn acquire(path: &Path, timeout: Duration) -> Result<Self, IoError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(IoError::from_io)?;
        let start = Instant::now();
        loop {
            match fs4::FileExt::try_lock_exclusive(&file) {
                Ok(()) => return Ok(Self { file }),
                Err(err) => {
                    if start.elapsed() >= timeout {
                        return Err(IoError::new(IoErrorKind::LockTimeout, err.to_string()));
                    }
                    std::thread::sleep(Duration::from_millis(25));
                }
            }
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = fs4::FileExt::unlock(&self.file);
    }
}

#[must_use]
pub fn normalize_run_base_dir(cwd: &Path, run_base: &Path) -> PathBuf {
    if run_base.is_absolute() {
        run_base.to_path_buf()
    } else {
        cwd.join(run_base)
    }
}

#[derive(Debug, Clone)]
pub struct RunLayoutPaths {
    pub run_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub tmp_dir: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub struct RunLayoutContract {
    pub runs_dir: &'static str,
    pub artifacts_dir: &'static str,
    pub logs_dir: &'static str,
    pub tmp_dir: &'static str,
    pub lock_file: &'static str,
    pub publish_marker: &'static str,
}

pub const RUN_LAYOUT_CONTRACT: RunLayoutContract = RunLayoutContract {
    runs_dir: "runs",
    artifacts_dir: "artifacts",
    logs_dir: "logs",
    tmp_dir: "tmp",
    lock_file: ".run.lock",
    publish_marker: "published.json",
};

pub const PIPELINE_RUN_DIR_TEMPLATE: &str = "{pipeline_id}/{sample_id}/{run_id}";

#[must_use]
pub fn pipeline_run_dir(
    base_dir: &Path,
    pipeline_id: &str,
    sample_id: &str,
    run_id: &str,
) -> PathBuf {
    base_dir.join(pipeline_id).join(sample_id).join(run_id)
}

#[must_use]
pub fn run_layout_paths(base_dir: &Path, run_id: &str) -> RunLayoutPaths {
    let run_dir = base_dir.join(RUN_LAYOUT_CONTRACT.runs_dir).join(run_id);
    RunLayoutPaths {
        artifacts_dir: run_dir.join(RUN_LAYOUT_CONTRACT.artifacts_dir),
        logs_dir: run_dir.join(RUN_LAYOUT_CONTRACT.logs_dir),
        tmp_dir: run_dir.join(RUN_LAYOUT_CONTRACT.tmp_dir),
        run_dir,
    }
}

#[must_use]
pub fn run_stage_dir(base_dir: &Path, run_id: &str, stage: &str, tool: &str) -> PathBuf {
    run_layout_paths(base_dir, run_id)
        .run_dir
        .join(stage)
        .join(tool)
}

/// Acquire the run-level lock for coordinated publish/write operations.
///
/// # Errors
/// Returns an IO error if the lock cannot be acquired within the timeout.
pub fn lock_run(layout: &RunLayoutPaths, timeout: Duration) -> Result<FileLock, IoError> {
    ensure_dir(&layout.run_dir)?;
    FileLock::acquire(&layout.run_dir.join(RUN_LAYOUT_CONTRACT.lock_file), timeout)
}

/// Publish a run by writing an atomic marker into the artifacts directory.
///
/// # Errors
/// Returns an IO error if the marker cannot be written.
pub fn publish_run(layout: &RunLayoutPaths, run_id: &str) -> Result<PathBuf, IoError> {
    ensure_dir(&layout.artifacts_dir)?;
    let marker = layout
        .artifacts_dir
        .join(RUN_LAYOUT_CONTRACT.publish_marker);
    let payload = serde_json::json!({
        "schema_version": "bijux.run_publish.v1",
        "run_id": run_id,
    });
    atomic_write_json(&marker, &payload)?;
    Ok(marker)
}

/// Create a managed temporary directory.
///
/// # Errors
/// Returns an IO error if the temp directory cannot be created.
pub fn temp_dir(prefix: &str) -> Result<tempfile::TempDir, IoError> {
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir()
        .map_err(IoError::from_io)
}

/// Create a managed temporary directory under a base path.
///
/// # Errors
/// Returns an IO error if the temp directory cannot be created.
pub fn temp_dir_in(base: &Path, prefix: &str) -> Result<tempfile::TempDir, IoError> {
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(base)
        .map_err(IoError::from_io)
}

/// Hash a file using SHA-256.
///
/// # Errors
/// Returns an IO error if the file cannot be read.
pub fn hash_file_sha256(path: &Path) -> Result<String, IoError> {
    use sha2::{Digest, Sha256};
    let mut file = File::open(path).map_err(IoError::from_io)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = file.read(&mut buffer).map_err(IoError::from_io)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[allow(clippy::unnecessary_wraps)]
fn default_permissions() -> Option<std::fs::Permissions> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        Some(std::fs::Permissions::from_mode(0o644))
    }
    #[cfg(not(unix))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

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
        let dir = crate::temp_dir("bijux")?;
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
        let dir = crate::temp_dir("bijux")?;
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
        let dir = crate::temp_dir("bijux")?;
        let layout = run_layout_paths(dir.path(), "run-1");
        let _lock = lock_run(&layout, Duration::from_millis(50))?;
        let marker = publish_run(&layout, "run-1")?;
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
        let dir = temp_dir("bijux-test")?;
        assert!(dir.path().exists());
        Ok(())
    }
}
