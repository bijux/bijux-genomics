//! Owner: runner age process backend.
use crate::command_runner::run_command_bytes_with_timeout;
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::time::Duration;
const AGE_COMMAND_TIMEOUT: Duration = Duration::from_mins(5);
use super::storage::with_staged_plaintext;
pub(super) fn encrypt_age_cli(
    recipients: &[String],
    plaintext: &[u8],
    output_path: &Path,
) -> Result<()> {
    if recipients.is_empty() {
        return Err(anyhow!("age-cli backend requires at least one recipient"));
    }
    with_staged_plaintext(output_path, plaintext, |plaintext_path| {
        let mut args = vec!["--encrypt".to_string()];
        for recipient in recipients {
            args.push("-r".to_string());
            args.push(recipient.clone());
        }
        args.push("-o".to_string());
        args.push(output_path.display().to_string());
        args.push(plaintext_path.display().to_string());

        let output = run_command_bytes_with_timeout("age", &args, AGE_COMMAND_TIMEOUT)
            .with_context(|| format!("run age --encrypt for {}", output_path.display()));
        let output = output?;
        if output.timed_out || !output.status.success() {
            return Err(anyhow!(
                "age --encrypt failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        Ok(())
    })
}
pub(super) fn decrypt_age_cli(bundle_path: &Path, identity_files: &[PathBuf]) -> Result<Vec<u8>> {
    if identity_files.is_empty() {
        return Err(anyhow!(
            "age-cli decrypt requires at least one identity file; pass --identity-file"
        ));
    }
    let mut args = vec!["--decrypt".to_string()];
    for identity_file in identity_files {
        args.push("-i".to_string());
        args.push(identity_file.display().to_string());
    }
    args.push(bundle_path.display().to_string());
    let output = run_command_bytes_with_timeout("age", &args, AGE_COMMAND_TIMEOUT)
        .with_context(|| format!("run age --decrypt for {}", bundle_path.display()))?;
    if output.timed_out || !output.status.success() {
        return Err(anyhow!(
            "age --decrypt failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}
