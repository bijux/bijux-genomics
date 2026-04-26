use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

static CWD_LOCK: Mutex<()> = Mutex::new(());
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn cwd_lock() -> MutexGuard<'static, ()> {
    CWD_LOCK.lock().unwrap_or_else(|err| panic!("cwd lock poisoned: {err}"))
}

fn env_lock() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|err| panic!("env lock poisoned: {err}"))
}

struct CurrentDirGuard {
    previous: PathBuf,
}

impl CurrentDirGuard {
    fn change_to(path: &Path) -> anyhow::Result<Self> {
        let previous = std::env::current_dir()?;
        std::env::set_current_dir(path)?;
        Ok(Self { previous })
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.previous);
    }
}

struct EnvVarGuard {
    key: &'static str,
    previous: Option<OsString>,
}

impl EnvVarGuard {
    fn set_path(key: &'static str, value: &Path) -> Self {
        let previous = std::env::var_os(key);
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(previous) = &self.previous {
            std::env::set_var(self.key, previous);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

#[test]
fn atomic_write_failure_does_not_clobber() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let target = dir.path().join("payload.json");
    bijux_dna_infra::atomic_write_bytes(&target, b"stable")?;
    let result =
        bijux_dna_infra::atomic_write_with(&target, |_| Err(std::io::Error::other("boom")));
    assert!(result.is_err());
    let data = std::fs::read_to_string(&target)?;
    assert_eq!(data, "stable");
    Ok(())
}

#[test]
fn leaf_relative_paths_write_in_current_directory() -> anyhow::Result<()> {
    let _cwd = cwd_lock();
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let _guard = CurrentDirGuard::change_to(dir.path())?;

    bijux_dna_infra::atomic_write_bytes(Path::new("payload.bin"), b"payload")?;
    assert_eq!(std::fs::read("payload.bin")?, b"payload");

    let mut file = bijux_dna_infra::create_file(Path::new("created.txt"))?;
    file.write_all(b"created")?;
    drop(file);
    assert_eq!(std::fs::read_to_string("created.txt")?, "created");
    Ok(())
}

#[cfg(unix)]
#[test]
fn atomic_write_preserves_existing_permissions() -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let path = dir.path().join("private.txt");
    bijux_dna_infra::atomic_write_bytes(&path, b"first")?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;

    bijux_dna_infra::atomic_write_bytes(&path, b"second")?;

    assert_eq!(std::fs::read_to_string(&path)?, "second");
    assert_eq!(std::fs::metadata(&path)?.permissions().mode() & 0o777, 0o600);
    Ok(())
}

#[test]
fn rename_creates_destination_parent() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let source = dir.path().join("source.txt");
    let destination = dir.path().join("nested").join("renamed.txt");
    bijux_dna_infra::atomic_write_bytes(&source, b"payload")?;

    bijux_dna_infra::rename(&source, &destination)?;

    assert!(!source.exists());
    assert_eq!(std::fs::read_to_string(destination)?, "payload");
    Ok(())
}

#[test]
fn copy_file_creates_destination_parent() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let source = dir.path().join("source.txt");
    let destination = dir.path().join("nested").join("copied.txt");
    bijux_dna_infra::atomic_write_bytes(&source, b"payload")?;

    let bytes = bijux_dna_infra::copy_file(&source, &destination)?;

    assert_eq!(bytes, 7);
    assert_eq!(std::fs::read_to_string(destination)?, "payload");
    Ok(())
}

#[test]
fn bounded_read_rejects_files_larger_than_limit() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let path = dir.path().join("payload.bin");
    bijux_dna_infra::atomic_write_bytes(&path, b"abcdef")?;

    let exact = bijux_dna_infra::read_to_end_bounded(&path, 6)?;
    assert_eq!(exact, b"abcdef");

    let err = bijux_dna_infra::read_to_end_bounded(&path, 5)
        .err()
        .ok_or_else(|| anyhow::anyhow!("expected bounded read failure"))?;
    assert_eq!(err.kind, bijux_dna_infra::IoErrorKind::Corruption);
    Ok(())
}

#[test]
fn path_shape_io_errors_are_corruption() {
    assert_eq!(
        bijux_dna_infra::classify_io_error(&std::io::Error::from(
            std::io::ErrorKind::AlreadyExists
        )),
        bijux_dna_infra::IoErrorKind::Corruption
    );
    assert_eq!(
        bijux_dna_infra::classify_io_error(&std::io::Error::from(
            std::io::ErrorKind::NotADirectory
        )),
        bijux_dna_infra::IoErrorKind::Corruption
    );
}

#[test]
fn json_serialization_errors_are_not_input_corruption() {
    let value = BTreeMap::from([((1_u8, 2_u8), "not-a-json-object-key")]);
    let err = bijux_dna_infra::atomic_write_json(Path::new("unused.json"), &value)
        .err()
        .unwrap_or_else(|| panic!("expected JSON serialization failure"));
    assert_eq!(err.kind, bijux_dna_infra::IoErrorKind::Other);
}

#[test]
fn temp_dir_is_created() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux-dna-test")?;
    assert!(dir.path().exists());
    Ok(())
}

#[test]
fn temp_dir_honors_test_tmp_dir() -> anyhow::Result<()> {
    let _env = env_lock();
    let base = tempfile::tempdir()?;
    let test_tmp = base.path().join("artifact-rooted-temp");
    let _guard = EnvVarGuard::set_path("TEST_TMP_DIR", &test_tmp);

    let dir = bijux_dna_infra::temp_dir("bijux-dna-test")?;

    assert!(dir.path().starts_with(&test_tmp));
    assert!(dir.path().exists());
    Ok(())
}

#[test]
fn temp_dir_in_creates_base_directory() -> anyhow::Result<()> {
    let base = tempfile::tempdir()?;
    let nested = base.path().join("missing").join("tmp");

    let dir = bijux_dna_infra::temp_dir_in(&nested, "bijux-dna-test")?;

    assert!(dir.path().starts_with(&nested));
    assert!(dir.path().exists());
    Ok(())
}

#[test]
fn logging_without_tracing_feature_returns_io_error() {
    let err = bijux_dna_infra::init_logging(Path::new("logs/bijux.log"))
        .err()
        .unwrap_or_else(|| panic!("expected logging setup to require tracing feature"));
    assert_eq!(err.kind, bijux_dna_infra::IoErrorKind::Other);
    assert!(err.message.contains("tracing feature"));
}

#[cfg(unix)]
#[test]
fn remove_path_if_exists_removes_broken_symlink() -> anyhow::Result<()> {
    use std::os::unix::fs::symlink;

    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let link = dir.path().join("dangling-link");
    symlink(dir.path().join("missing-target"), &link)?;

    bijux_dna_infra::remove_path_if_exists(&link)?;

    assert!(std::fs::symlink_metadata(&link).is_err(), "broken symlink should be removed");
    Ok(())
}
