use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::Result;

pub static CWD_LOCK: Mutex<()> = Mutex::new(());

pub struct EnvGuard {
    cwd: PathBuf,
    env: BTreeMap<OsString, OsString>,
}

impl EnvGuard {
    pub fn new() -> Result<Self> {
        Ok(Self {
            cwd: std::env::current_dir()?,
            env: std::env::vars_os().collect(),
        })
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

pub fn with_repo_root<F, T>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    let env_guard = EnvGuard::new()?;
    let _guard = CWD_LOCK.lock().expect("cwd lock");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("repo root not found"))?;
    let prev_dir = std::env::current_dir()?;
    std::env::set_current_dir(repo_root)?;
    let result = f();
    std::env::set_current_dir(prev_dir)?;
    drop(env_guard);
    result
}
