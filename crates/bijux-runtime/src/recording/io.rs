use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use sha2::Digest;

/// Append a line to a JSONL file (create if missing).
///
/// # Errors
/// Returns an error if the file cannot be opened or written.
pub fn append_jsonl_line(path: &Path, line: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{line}")?;
    Ok(())
}

/// Write bytes atomically by writing a temp file and renaming.
///
/// # Errors
/// Returns an error if the target cannot be written.
pub fn write_atomic_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| anyhow!("missing parent for {}", path.display()))?;
    bijux_infra::ensure_dir(dir)?;
    let mut temp = PathBuf::from(path);
    temp.set_extension("tmp");
    let mut file = std::fs::File::create(&temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    bijux_infra::rename(&temp, path)?;
    Ok(())
}

/// Write canonical JSON using core canonicalizer.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_canonical_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let payload = bijux_core::contract::canonical::to_canonical_json_bytes(value)?;
    write_atomic_bytes(path, payload.as_slice())
}

/// # Errors
/// Returns an error if execution logs cannot be written.
pub fn write_execution_logs(run_logs_dir: &Path, stdout: &str, stderr: &str) -> Result<()> {
    let _ = write_execution_logs_bounded(run_logs_dir, stdout, stderr)?;
    Ok(())
}

/// # Errors
/// Returns an error if bounded execution logs cannot be written.
pub fn write_execution_logs_bounded(
    logs_dir: &Path,
    stdout: &str,
    stderr: &str,
) -> Result<Vec<PathBuf>> {
    bijux_infra::ensure_dir(logs_dir).context("create logs dir")?;
    let tail_kb = log_tail_kb();
    let stdout_path = logs_dir.join("tool.stdout.log");
    let stderr_path = logs_dir.join("tool.stderr.log");
    let combined_path = logs_dir.join("tool.log");
    let stdout_tail = truncate_tail(stdout, tail_kb);
    let stderr_tail = truncate_tail(stderr, tail_kb);
    write_atomic_bytes(&stdout_path, stdout_tail.as_bytes()).context("write tool.stdout.log")?;
    write_atomic_bytes(&stderr_path, stderr_tail.as_bytes()).context("write tool.stderr.log")?;
    let combined = if stderr.is_empty() {
        truncate_tail(stdout, tail_kb)
    } else {
        truncate_tail(&format!("{stdout}\n--- stderr ---\n{stderr}"), tail_kb)
    };
    write_atomic_bytes(&combined_path, combined.as_bytes()).context("write tool.log")?;
    Ok(vec![combined_path, stdout_path, stderr_path])
}

/// # Errors
/// Returns an error if hashing fails.
pub fn hash_file_sha256(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = sha2::Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let read = std::io::Read::read(&mut file, &mut buf)
            .with_context(|| format!("read {}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn log_tail_kb() -> usize {
    std::env::var("BIJUX_LOG_TAIL_KB")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map_or(128, |value| value.clamp(1, 4096))
}

fn truncate_tail(text: &str, tail_kb: usize) -> String {
    let max_bytes = tail_kb.saturating_mul(1024);
    if text.len() <= max_bytes {
        return text.to_string();
    }
    let bytes = text.as_bytes();
    let start = bytes.len().saturating_sub(max_bytes);
    String::from_utf8_lossy(&bytes[start..]).to_string()
}
