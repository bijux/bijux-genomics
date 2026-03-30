use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sha2::Digest as _;

use crate::commands::cli;

pub(crate) struct CwdGuard(PathBuf);

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}

/// # Errors
/// Returns an error if the process current directory cannot be set.
pub(crate) fn enter_cli_cwd(cwd: &Path) -> Result<CwdGuard> {
    let original_cwd = std::env::current_dir().context("resolve current dir")?;
    std::env::set_current_dir(cwd).context("set current dir")?;
    Ok(CwdGuard(original_cwd))
}

pub(crate) fn configure_process_cli_env(cli: &cli::Cli, cwd: &Path) {
    if let Some(path) = &cli.telemetry_jsonl {
        let telemetry_path = if path.is_absolute() {
            path.clone()
        } else {
            cwd.join(path)
        };
        std::env::set_var("BIJUX_TELEMETRY_JSONL", telemetry_path);
    }
    if cli.json {
        std::env::set_var("BIJUX_OUTPUT_JSON", "1");
    }
    if cli.verbose {
        std::env::set_var("BIJUX_VERBOSE", "1");
    }
    if cli.quiet {
        std::env::set_var("BIJUX_QUIET", "1");
    }
    if let Some(level) = &cli.log_level {
        std::env::set_var("RUST_LOG", level);
    }
}

/// # Errors
/// Returns an error if the runtime profile cannot be serialized.
pub(crate) fn configure_run_context_env<T>(cli: &cli::Cli, profile: &T) -> Result<()>
where
    T: serde::Serialize,
{
    let bytes = serde_json::to_vec(profile)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    std::env::set_var("BIJUX_PROFILE_HASH", format!("{:x}", hasher.finalize()));
    if cli.profile.eq_ignore_ascii_case("hpc") {
        std::env::set_var("BIJUX_RUN_CONTEXT", "hpc");
        if std::env::var("BIJUX_HPC_SITE")
            .ok()
            .map_or(true, |value| value.trim().is_empty())
        {
            if let Some(platform) = std::env::var("BIJUX_PLATFORM")
                .ok()
                .filter(|value| !value.trim().is_empty())
            {
                std::env::set_var("BIJUX_HPC_SITE", platform);
            }
        }
    } else {
        std::env::set_var("BIJUX_RUN_CONTEXT", "local");
    }
    Ok(())
}
