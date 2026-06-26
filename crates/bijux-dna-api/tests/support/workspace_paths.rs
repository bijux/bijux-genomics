#![allow(dead_code)]

use anyhow::{anyhow, Result};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
#[cfg(unix)]
use sysinfo::{Pid, System};

fn looks_like_repo_root(path: &Path) -> bool {
    path.join("Cargo.lock").is_file()
        && path.join("crates").is_dir()
        && path.join("configs").is_dir()
}

/// Resolve the workspace root used by crate test support helpers.
///
/// # Errors
///
/// Returns an error when the current directory cannot be read or no ancestor
/// matches the expected repository layout.
pub fn repo_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir().map_err(|err| anyhow!("resolve current directory: {err}"))?;
    for candidate in cwd.ancestors() {
        if looks_like_repo_root(candidate) {
            return Ok(candidate.to_path_buf());
        }
    }
    Err(anyhow!("unable to resolve repository root from {}", cwd.display()))
}

/// Resolve the crate root for repository-scoped fixtures and snapshots.
///
/// # Errors
///
/// Propagates any repository root resolution failure.
pub fn crate_root(crate_name: &str) -> Result<PathBuf> {
    Ok(repo_root()?.join("crates").join(crate_name))
}

/// Resolve the `src` directory for a crate under test.
///
/// # Errors
///
/// Propagates any repository root resolution failure.
pub fn crate_src(crate_name: &str) -> Result<PathBuf> {
    Ok(crate_root(crate_name)?.join("src"))
}

/// Resolve the snapshot directory for a crate under test.
///
/// # Errors
///
/// Propagates any repository root resolution failure.
pub fn crate_snapshots(crate_name: &str) -> Result<PathBuf> {
    Ok(crate_root(crate_name)?.join("tests").join("snapshots"))
}

const TEST_LOCK_ROOT: &str = "artifacts/test-locks";
const TEST_LOCK_WAIT_TIMEOUT: Duration = Duration::from_mins(5);
const TEST_LOCK_POLL_INTERVAL: Duration = Duration::from_millis(50);
const TEST_LOCK_OWNER_FILE: &str = "owner.pid";
const TEST_LOCK_MISSING_OWNER_GRACE: Duration = Duration::from_secs(1);

pub struct RepoProcessLock {
    path: PathBuf,
}

impl RepoProcessLock {
    pub fn acquire(name: &str) -> Result<Self> {
        let repo_root = repo_root()?;
        let lock_root = repo_root.join(TEST_LOCK_ROOT);
        fs::create_dir_all(&lock_root)
            .map_err(|err| anyhow!("create repo test lock root {}: {err}", lock_root.display()))?;
        let path = lock_root.join(name);
        let deadline = Instant::now() + TEST_LOCK_WAIT_TIMEOUT;

        loop {
            match fs::create_dir(&path) {
                Ok(()) => {
                    write_lock_owner(&path)?;
                    return Ok(Self { path });
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                    if stale_repo_test_lock(&path)? {
                        match fs::remove_dir_all(&path) {
                            Ok(()) => continue,
                            Err(remove_error) if remove_error.kind() == ErrorKind::NotFound => {
                                continue;
                            }
                            Err(remove_error) => {
                                return Err(anyhow!(
                                    "remove stale repo test lock `{}`: {remove_error}",
                                    path.display()
                                ));
                            }
                        }
                    }
                    if Instant::now() >= deadline {
                        return Err(anyhow!(
                            "timed out waiting for repo test lock `{}`",
                            path.display()
                        ));
                    }
                    std::thread::sleep(TEST_LOCK_POLL_INTERVAL);
                }
                Err(error) => {
                    return Err(anyhow!("create repo test lock `{}`: {error}", path.display()));
                }
            }
        }
    }
}

impl Drop for RepoProcessLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub fn bench_output_lock() -> Result<RepoProcessLock> {
    RepoProcessLock::acquire("bijux-dna-api-bench-output")
}

fn write_lock_owner(path: &Path) -> Result<()> {
    fs::write(path.join(TEST_LOCK_OWNER_FILE), std::process::id().to_string())
        .map_err(|error| anyhow!("write repo test lock owner `{}`: {error}", path.display()))
}

fn stale_repo_test_lock(path: &Path) -> Result<bool> {
    let owner_path = path.join(TEST_LOCK_OWNER_FILE);
    match fs::read_to_string(&owner_path) {
        Ok(raw_pid) => {
            let pid = raw_pid.trim().parse::<u32>().map_err(|error| {
                anyhow!("parse repo test lock owner `{}`: {error}", owner_path.display())
            })?;
            Ok(!process_is_alive(pid))
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            Ok(lock_is_older_than(path, TEST_LOCK_MISSING_OWNER_GRACE)?)
        }
        Err(error) => Err(anyhow!("read repo test lock owner `{}`: {error}", owner_path.display())),
    }
}

fn lock_is_older_than(path: &Path, threshold: Duration) -> Result<bool> {
    let modified = match fs::metadata(path) {
        Ok(metadata) => metadata.modified().map_err(|error| {
            anyhow!("read repo test lock metadata `{}`: {error}", path.display())
        })?,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(true),
        Err(error) => {
            return Err(anyhow!("read repo test lock metadata `{}`: {error}", path.display()));
        }
    };
    let age = modified
        .elapsed()
        .map_err(|error| anyhow!("measure repo test lock age `{}`: {error}", path.display()))?;
    Ok(age >= threshold)
}

#[cfg(unix)]
fn process_is_alive(pid: u32) -> bool {
    let system = System::new_all();
    system.process(Pid::from_u32(pid)).is_some()
}

#[cfg(not(unix))]
fn process_is_alive(_pid: u32) -> bool {
    true
}
