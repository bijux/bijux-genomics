#![cfg(unix)]

use std::fs;
use std::io::Write as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context as _, Result};

static PUBLICATION_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) fn write_executable(path: &Path, body: &str) -> Result<()> {
    let parent = path.parent().context("scheduler executable has no parent directory")?;
    let file_name = path.file_name().context("scheduler executable has no file name")?;
    let sequence = PUBLICATION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let staging = parent.join(format!(
        ".{}.publish-{}-{sequence}",
        file_name.to_string_lossy(),
        std::process::id()
    ));

    let result = (|| -> Result<()> {
        let mut file =
            fs::OpenOptions::new().write(true).create_new(true).open(&staging).with_context(
                || format!("create scheduler executable staging file {}", staging.display()),
            )?;
        file.write_all(body.as_bytes())?;
        let mut permissions = file.metadata()?.permissions();
        permissions.set_mode(0o755);
        file.set_permissions(permissions)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&staging, path).with_context(|| {
            format!("publish scheduler executable {} from {}", path.display(), staging.display())
        })?;
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&staging);
    }
    result
}
