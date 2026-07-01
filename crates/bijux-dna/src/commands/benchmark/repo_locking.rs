use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};

const TEST_LOCK_ROOT: &str = "artifacts/test-locks";
const TEST_LOCK_WAIT_TIMEOUT: Duration = Duration::from_mins(5);
const TEST_LOCK_POLL_INTERVAL: Duration = Duration::from_millis(50);
const TEST_LOCK_OWNER_FILE: &str = "owner.pid";
const TEST_LOCK_MISSING_OWNER_GRACE: Duration = Duration::from_secs(1);

static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub(crate) fn cwd_lock() -> &'static Mutex<()> {
    CWD_LOCK.get_or_init(|| Mutex::new(()))
}

pub(crate) fn acquire_cwd_lock() -> MutexGuard<'static, ()> {
    cwd_lock().lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(crate) struct RepoProcessLock {
    path: PathBuf,
}

impl RepoProcessLock {
    pub(crate) fn acquire(repo_root: &Path, name: &str) -> Result<Self> {
        let lock_root = repo_root.join(TEST_LOCK_ROOT);
        fs::create_dir_all(&lock_root)?;
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

fn write_lock_owner(path: &Path) -> Result<()> {
    fs::write(path.join(TEST_LOCK_OWNER_FILE), std::process::id().to_string())
        .map_err(|error| anyhow!("write repo test lock owner `{}`: {error}", path.display()))
}

fn stale_repo_test_lock(path: &Path) -> Result<bool> {
    let owner_path = path.join(TEST_LOCK_OWNER_FILE);
    match fs::read_to_string(&owner_path) {
        Ok(raw_pid) => match raw_pid.trim().parse::<u32>() {
            Ok(pid) => Ok(!process_is_alive(pid)),
            Err(_) => Ok(lock_is_older_than(path, TEST_LOCK_MISSING_OWNER_GRACE)?),
        },
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
    let Ok(pid) = i32::try_from(pid) else {
        return false;
    };
    match nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), None) {
        Ok(()) | Err(nix::errno::Errno::EPERM) => true,
        Err(_) => false,
    }
}

#[cfg(not(unix))]
fn process_is_alive(_pid: u32) -> bool {
    true
}
