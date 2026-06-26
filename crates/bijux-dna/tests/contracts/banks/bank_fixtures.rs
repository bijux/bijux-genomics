use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use serde_json::{Map, Value};

#[path = "../../support/workspace_paths.rs"]
mod test_support;

pub static CWD_LOCK: Mutex<()> = Mutex::new(());
#[allow(dead_code)]
const TEST_LOCK_ROOT: &str = "artifacts/test-locks";
#[allow(dead_code)]
const TEST_LOCK_WAIT_TIMEOUT: Duration = Duration::from_mins(5);
#[allow(dead_code)]
const TEST_LOCK_POLL_INTERVAL: Duration = Duration::from_millis(50);
#[allow(dead_code)]
const TEST_LOCK_OWNER_FILE: &str = "owner.pid";
#[allow(dead_code)]
const TEST_LOCK_MISSING_OWNER_GRACE: Duration = Duration::from_secs(1);

pub struct EnvGuard {
    cwd: PathBuf,
    env: BTreeMap<OsString, OsString>,
}

impl EnvGuard {
    pub fn new() -> Result<Self> {
        Ok(Self { cwd: std::env::current_dir()?, env: std::env::vars_os().collect() })
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        let current: BTreeMap<OsString, OsString> = std::env::vars_os().collect();
        for key in current.keys() {
            if !self.env.contains_key(key) {
                std::env::remove_var(key);
            }
        }
        for (key, value) in &self.env {
            std::env::set_var(key, value);
        }
        let _ = std::env::set_current_dir(&self.cwd);
    }
}

#[allow(dead_code)]
pub fn crate_root(name: &str) -> Result<PathBuf> {
    test_support::crate_root(name)
}

pub fn repo_root() -> Result<PathBuf> {
    test_support::repo_root()
}

#[allow(dead_code)]
pub struct RepoProcessLock {
    path: PathBuf,
}

#[allow(dead_code)]
impl RepoProcessLock {
    pub fn acquire(name: &str) -> Result<Self> {
        let repo_root = test_support::repo_root()?;
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
    let system = sysinfo::System::new_all();
    system.process(sysinfo::Pid::from_u32(pid)).is_some()
}

#[cfg(not(unix))]
fn process_is_alive(_pid: u32) -> bool {
    true
}

#[allow(dead_code)]
pub fn with_repo_root<F, T>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    let _guard = CWD_LOCK.lock().map_err(|err| anyhow!("cwd lock: {err}"))?;
    let _env_guard = EnvGuard::new()?;
    let repo_root = test_support::repo_root()?;
    std::env::set_current_dir(&repo_root)?;
    f()
}

#[allow(dead_code)]
pub fn json_u64(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

#[allow(dead_code)]
pub fn json_bool(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

#[allow(dead_code)]
pub fn json_object<'a>(value: &'a Value, key: &str) -> &'a Map<String, Value> {
    value.get(key).and_then(Value::as_object).expect(key)
}

#[allow(dead_code)]
pub fn json_array<'a>(value: &'a Value, key: &str) -> &'a Vec<Value> {
    value.get(key).and_then(Value::as_array).expect(key)
}

#[allow(dead_code)]
pub fn object_u64(map: &Map<String, Value>, key: &str) -> Option<u64> {
    map.get(key).and_then(Value::as_u64)
}

#[allow(dead_code)]
pub fn object_u64_sum(map: &Map<String, Value>) -> u64 {
    map.values().filter_map(Value::as_u64).sum()
}
