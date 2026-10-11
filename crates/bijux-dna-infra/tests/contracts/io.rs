use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

static CWD_LOCK: Mutex<()> = Mutex::new(());

fn cwd_lock() -> MutexGuard<'static, ()> {
    CWD_LOCK.lock().unwrap_or_else(|err| panic!("cwd lock poisoned: {err}"))
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
fn atomic_create_refuses_to_replace_existing_data() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let target = dir.path().join("immutable.json");
    bijux_dna_infra::atomic_create_bytes(&target, b"first")?;

    let error = bijux_dna_infra::atomic_create_bytes(&target, b"second")
        .err()
        .ok_or_else(|| anyhow::anyhow!("expected immutable destination collision"))?;

    assert_eq!(error.kind, bijux_dna_infra::IoErrorKind::Corruption);
    assert_eq!(std::fs::read(&target)?, b"first");
    Ok(())
}

#[test]
fn atomic_create_failure_does_not_publish_partial_data() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let target = dir.path().join("immutable.json");

    let result = bijux_dna_infra::atomic_create_with(&target, |file| {
        file.write_all(b"partial")?;
        Err(std::io::Error::other("boom"))
    });

    assert!(result.is_err());
    assert!(!target.exists());
    Ok(())
}

#[test]
fn atomic_create_json_publishes_complete_payload() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let target = dir.path().join("immutable.json");
    let payload = BTreeMap::from([("state", "planned")]);

    bijux_dna_infra::atomic_create_json(&target, &payload)?;

    let observed: BTreeMap<String, String> = serde_json::from_slice(&std::fs::read(target)?)?;
    assert_eq!(observed.get("state").map(String::as_str), Some("planned"));
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
fn temp_dir_in_creates_base_directory() -> anyhow::Result<()> {
    let base = tempfile::tempdir()?;
    let nested = base.path().join("missing").join("tmp");

    let dir = bijux_dna_infra::temp_dir_in(&nested, "bijux-dna-test")?;

    assert!(dir.path().starts_with(&nested));
    assert!(dir.path().exists());
    Ok(())
}

#[test]
fn relative_test_tmp_dir_is_independent_of_current_directory() -> anyhow::Result<()> {
    let Some(test_tmp_dir) = std::env::var_os("TEST_TMP_DIR").map(PathBuf::from) else {
        return Ok(());
    };
    let _cwd = cwd_lock();
    let workspace_root = workspace_root();
    let dir = tempfile::tempdir()?;
    let _guard = CurrentDirGuard::change_to(dir.path())?;

    let temp = bijux_dna_infra::temp_dir("bijux-dna-test")?;

    let expected_root =
        if test_tmp_dir.is_absolute() { test_tmp_dir } else { workspace_root.join(test_tmp_dir) };
    assert!(temp.path().starts_with(expected_root));
    Ok(())
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|crates_dir| crates_dir.parent())
        .unwrap_or_else(|| panic!("resolve workspace root from CARGO_MANIFEST_DIR"))
        .to_path_buf()
}

#[test]
#[cfg(not(feature = "tracing"))]
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

#[test]
fn empty_directory_removal_preserves_unexpected_contents() -> anyhow::Result<()> {
    let scratch = bijux_dna_infra::temp_dir("empty-directory-")?;
    let directory = scratch.path().join("journal");
    bijux_dna_infra::ensure_dir(&directory)?;
    let evidence = directory.join("unexpected");
    bijux_dna_infra::write_bytes(&evidence, b"retain me")?;
    assert!(bijux_dna_infra::remove_empty_dir(&directory).is_err());
    assert_eq!(std::fs::read(&evidence)?, b"retain me");
    bijux_dna_infra::remove_file(&evidence)?;
    bijux_dna_infra::remove_empty_dir(&directory)?;
    assert!(!directory.exists());
    Ok(())
}

#[test]
fn private_atomic_publication_replaces_bytes_and_cleans_failed_staging() -> anyhow::Result<()> {
    let root = bijux_dna_infra::temp_dir("bijux-private-publication")?;
    let output = root.path().join("plaintext.json");
    bijux_dna_infra::atomic_write_private_bytes(&output, b"first plaintext")?;
    assert_eq!(std::fs::read(&output)?, b"first plaintext");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(std::fs::metadata(&output)?.permissions().mode() & 0o777, 0o600);
        std::fs::set_permissions(&output, std::fs::Permissions::from_mode(0o644))?;
    }
    bijux_dna_infra::atomic_write_private_bytes(&output, b"replacement plaintext")?;
    assert_eq!(std::fs::read(&output)?, b"replacement plaintext");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(std::fs::metadata(&output)?.permissions().mode() & 0o777, 0o600);
    }
    let directory = root.path().join("existing-directory");
    std::fs::create_dir(&directory)?;
    std::fs::write(directory.join("guard"), b"untouched")?;
    assert!(bijux_dna_infra::atomic_write_private_bytes(&directory, b"must not publish").is_err());
    assert_eq!(std::fs::read(directory.join("guard"))?, b"untouched");
    let mut entries = std::fs::read_dir(root.path())?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort();
    assert_eq!(
        entries,
        [
            std::ffi::OsString::from("existing-directory"),
            std::ffi::OsString::from("plaintext.json")
        ]
    );
    Ok(())
}
