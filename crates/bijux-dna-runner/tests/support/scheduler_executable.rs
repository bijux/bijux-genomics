#![cfg(unix)]

use std::fs;
use std::io::Write as _;
use std::os::unix::fs::{symlink, PermissionsExt as _};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context as _, Result};

static PUBLICATION_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) fn write_executable(path: &Path, body: &str) -> Result<()> {
    let parent = path.parent().context("scheduler executable has no parent directory")?;
    let file_name = path.file_name().context("scheduler executable has no file name")?;
    let file_name_text = file_name.to_string_lossy();
    let uses_stable_launcher = ["sbatch", "squeue", "sacct", "scancel"]
        .iter()
        .any(|command| file_name_text.starts_with(command));
    let body_path = parent.join(format!("{}.body", file_name.to_string_lossy()));
    let launcher = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/support/scheduler_executable_launcher.sh");
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
        if !uses_stable_launcher {
            let mut permissions = file.metadata()?.permissions();
            permissions.set_mode(0o755);
            file.set_permissions(permissions)?;
        }
        file.sync_all()?;
        drop(file);
        let publication_path = if uses_stable_launcher { &body_path } else { path };
        fs::rename(&staging, publication_path).with_context(|| {
            format!(
                "publish scheduler executable material {} from {}",
                publication_path.display(),
                staging.display()
            )
        })?;
        if uses_stable_launcher && !path.exists() {
            symlink(&launcher, path).with_context(|| {
                format!("link scheduler executable {} to {}", path.display(), launcher.display())
            })?;
        }
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&staging);
    }
    result
}
