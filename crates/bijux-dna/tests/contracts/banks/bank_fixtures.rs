use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{anyhow, Result};
use serde_json::{Map, Value};

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

#[allow(dead_code)]
pub fn crate_root(name: &str) -> Result<PathBuf> {
    test_support::crate_root(name)
}

pub fn repo_root() -> Result<PathBuf> {
    test_support::repo_root()
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
