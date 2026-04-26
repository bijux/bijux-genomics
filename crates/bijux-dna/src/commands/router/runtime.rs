use std::ffi::OsString;
use std::fmt::Write as _;
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

pub(crate) struct ProcessEnvGuard(Vec<(&'static str, Option<OsString>)>);

impl ProcessEnvGuard {
    pub(crate) fn capture(keys: &[&'static str]) -> Self {
        Self(keys.iter().map(|key| (*key, std::env::var_os(key))).collect())
    }
}

impl Drop for ProcessEnvGuard {
    fn drop(&mut self) {
        for (key, value) in self.0.iter().rev() {
            if let Some(value) = value {
                std::env::set_var(key, value);
            } else {
                std::env::remove_var(key);
            }
        }
    }
}

pub(crate) fn capture_cli_env() -> ProcessEnvGuard {
    ProcessEnvGuard::capture(&[
        "BIJUX_ALLOW_NETWORK",
        "BIJUX_ALLOW_SILVER",
        "BIJUX_EXPERIMENTAL_TOOLS",
        "BIJUX_HPC_SITE",
        "BIJUX_OUTPUT_JSON",
        "BIJUX_PLATFORM",
        "BIJUX_POLICY_CLEAN_REPORT_JSON",
        "BIJUX_PROFILE_HASH",
        "BIJUX_QUIET",
        "BIJUX_RUN_CONTEXT",
        "BIJUX_SCIENTIFIC_PRESET",
        "BIJUX_TELEMETRY_JSONL",
        "BIJUX_VERBOSE",
        "RUST_LOG",
    ])
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
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
        let telemetry_path = if path.is_absolute() { path.clone() } else { cwd.join(path) };
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
    if let Some(platform) = cli.platform.as_deref().map(str::trim).filter(|value| !value.is_empty())
    {
        std::env::set_var("BIJUX_PLATFORM", platform);
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
    std::env::set_var("BIJUX_PROFILE_HASH", sha256_hex(&hasher.finalize()));
    if cli.profile.eq_ignore_ascii_case("hpc") {
        std::env::set_var("BIJUX_RUN_CONTEXT", "hpc");
        if std::env::var("BIJUX_HPC_SITE").ok().is_none_or(|value| value.trim().is_empty()) {
            if let Some(platform) = cli
                .platform
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .or_else(|| {
                    std::env::var("BIJUX_PLATFORM")
                        .ok()
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty())
                })
            {
                std::env::set_var("BIJUX_HPC_SITE", platform);
            }
        }
    } else {
        std::env::set_var("BIJUX_RUN_CONTEXT", "local");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::sync::Mutex;

    use clap::Parser;

    use super::{capture_cli_env, configure_process_cli_env, configure_run_context_env};
    use crate::commands::cli;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn parse_cli(args: &[&str]) -> cli::Cli {
        cli::Cli::parse_from(std::iter::once("bijux-dna").chain(args.iter().copied()))
    }

    #[test]
    fn configure_process_cli_env_sets_and_restores_platform() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        std::env::set_var("BIJUX_PLATFORM", "outer-platform");
        {
            let _guard = capture_cli_env();
            let cli = parse_cli(&["--platform", "inner-platform", "env", "list"]);

            configure_process_cli_env(&cli, std::path::Path::new("."));

            assert_eq!(std::env::var("BIJUX_PLATFORM").as_deref(), Ok("inner-platform"));
        }

        assert_eq!(std::env::var("BIJUX_PLATFORM").as_deref(), Ok("outer-platform"));
    }

    #[test]
    fn hpc_run_context_prefers_cli_platform_for_site_identity() {
        let _lock = ENV_LOCK.lock().expect("env lock");
        let _guard = capture_cli_env();
        std::env::remove_var("BIJUX_HPC_SITE");
        std::env::set_var("BIJUX_PLATFORM", "env-platform");
        let cli = parse_cli(&["--profile", "hpc", "--platform", "cli-platform", "env", "list"]);

        configure_run_context_env(&cli, &serde_json::json!({"profile": "hpc"}))
            .expect("configure run context");

        assert_eq!(std::env::var("BIJUX_RUN_CONTEXT").as_deref(), Ok("hpc"));
        assert_eq!(std::env::var("BIJUX_HPC_SITE").as_deref(), Ok("cli-platform"));
    }
}
