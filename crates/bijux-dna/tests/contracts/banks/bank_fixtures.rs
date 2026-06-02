use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{anyhow, Result};

#[path = "../../support/workspace_paths.rs"]
mod test_support;

pub static CWD_LOCK: Mutex<()> = Mutex::new(());

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

pub fn crate_root(name: &str) -> Result<PathBuf> {
    test_support::crate_root(name)
}

pub fn repo_root() -> Result<PathBuf> {
    test_support::repo_root()
}

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
