use std::cell::Cell;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use anyhow::{anyhow, Result};
use serde_json::{Map, Value};

#[path = "../../support/workspace_paths.rs"]
mod test_support;

pub static CWD_LOCK: Mutex<()> = Mutex::new(());
#[allow(dead_code)]
const TEST_LOCK_ROOT: &str = "artifacts/test-locks";

thread_local! {
    static REPO_MUTATOR_LOCK_DEPTH: Cell<usize> = const { Cell::new(0) };
}

pub struct EnvGuard {
    cwd: PathBuf,
    env: BTreeMap<OsString, OsString>,
    _sandbox: Option<RepoSandbox>,
}

impl EnvGuard {
    pub fn new() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        let env = std::env::vars_os().collect();
        let sandbox = if repo_mutator_lock_held() {
            Some(RepoSandbox::new("locked-repository-mutator-")?)
        } else {
            None
        };
        if let Some(sandbox) = &sandbox {
            std::env::set_current_dir(sandbox.path())?;
            for (key, value) in sandbox.environment() {
                std::env::set_var(key, value);
            }
        }
        Ok(Self { cwd, env, _sandbox: sandbox })
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

#[allow(dead_code)]
pub fn repo_root() -> Result<PathBuf> {
    test_support::repo_root()
}

#[allow(dead_code)]
pub fn configure_isolated_runtime_resolution(command: &mut Command, root: &Path) {
    command
        .env("BIJUX_HPC_ROOT", root)
        .env("BIJUX_CACHE_ROOT", root.join("cache"))
        .env("BIJUX_APPTAINER_CONTAINER_DIR", root.join("containers"))
        .env("PATH", root.join("bin"));
}

#[allow(dead_code)]
pub fn hpc_dry_run_test_directory(repo_root: &Path, label: &str) -> Result<tempfile::TempDir> {
    let parent = repo_root.join("artifacts/runs/bench/hpc-dry-run");
    fs::create_dir_all(&parent)?;
    Ok(tempfile::Builder::new().prefix(label).tempdir_in(parent)?)
}

#[path = "../support/repository_sandbox/mod.rs"]
mod repository_sandbox;

// Each integration target consumes its own subset of these shared support helpers.
#[allow(unused_imports)]
pub use repository_sandbox::{bijux_dna_command, RepoSandbox};

#[allow(dead_code)]
#[must_use]
pub fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .map_or_else(|_| path.display().to_string(), |relative| relative.display().to_string())
        .replace('\\', "/")
}

#[allow(dead_code)]
pub struct RepoProcessLock {
    _file_lock: bijux_dna_infra::FileLock,
    _sandbox_request: RepoSandboxRequest,
}

#[allow(dead_code)]
impl RepoProcessLock {
    pub fn acquire(name: &str) -> Result<Self> {
        let repo_root = test_support::repo_root()?;
        let path = repo_root.join(TEST_LOCK_ROOT).join(format!("{name}.lock"));
        let file_lock = bijux_dna_infra::FileLock::try_acquire(&path)
            .map_err(|error| anyhow!("acquire repo test lock `{}`: {error}", path.display()))?;
        Ok(Self { _file_lock: file_lock, _sandbox_request: RepoSandboxRequest::new() })
    }
}

struct RepoSandboxRequest;

impl RepoSandboxRequest {
    fn new() -> Self {
        REPO_MUTATOR_LOCK_DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
        Self
    }
}

impl Drop for RepoSandboxRequest {
    fn drop(&mut self) {
        REPO_MUTATOR_LOCK_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

fn repo_mutator_lock_held() -> bool {
    REPO_MUTATOR_LOCK_DEPTH.with(|depth| depth.get() > 0)
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
pub fn json_object<'a>(value: &'a Value, key: &str) -> Result<&'a Map<String, Value>> {
    value
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("JSON field `{key}` must be an object"))
}

#[allow(dead_code)]
pub fn json_array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("JSON field `{key}` must be an array"))
}

#[allow(dead_code)]
pub fn object_u64(map: &Map<String, Value>, key: &str) -> Option<u64> {
    map.get(key).and_then(Value::as_u64)
}

#[allow(dead_code)]
pub fn object_u64_sum(map: &Map<String, Value>) -> u64 {
    map.values().filter_map(Value::as_u64).sum()
}
