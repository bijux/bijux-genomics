use std::path::Path;
use std::sync::{Mutex, MutexGuard, OnceLock};

use anyhow::{anyhow, Result};

const TEST_LOCK_ROOT: &str = "artifacts/test-locks";

static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub(crate) fn cwd_lock() -> &'static Mutex<()> {
    CWD_LOCK.get_or_init(|| Mutex::new(()))
}

pub(crate) fn acquire_cwd_lock() -> MutexGuard<'static, ()> {
    cwd_lock().lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(crate) struct RepoProcessLock {
    _file_lock: bijux_dna_infra::FileLock,
}

impl RepoProcessLock {
    pub(crate) fn acquire(repo_root: &Path, name: &str) -> Result<Self> {
        let path = repo_root.join(TEST_LOCK_ROOT).join(format!("{name}.lock"));
        let file_lock = bijux_dna_infra::FileLock::try_acquire(&path)
            .map_err(|error| anyhow!("acquire repo test lock `{}`: {error}", path.display()))?;
        Ok(Self { _file_lock: file_lock })
    }
}
