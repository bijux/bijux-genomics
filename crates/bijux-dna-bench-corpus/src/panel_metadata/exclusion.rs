//! Corpus panel metadata exclusion.
use anyhow::anyhow;
use anyhow::Result;
use std::sync::Mutex;
use std::sync::OnceLock;

/// Resolve or materialize the governed local panel metadata fixture.
///
/// # Errors
/// Returns an error for invalid reference authority, incompatible assets, poisoned locks or failed IO.
pub fn with_local_vcf_panel_fixture_lock<T>(action: impl FnOnce() -> Result<T>) -> Result<T> {
    static MATERIALIZATION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let lock = MATERIALIZATION_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().map_err(|_| anyhow!("local VCF panel fixture lock is poisoned"))?;
    action()
}
