use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::Result;

pub static CWD_LOCK: Mutex<()> = Mutex::new(());

pub fn with_repo_root<F, T>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
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
    result
}
