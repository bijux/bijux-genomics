use std::io::{self, Write};
use std::time::Instant;

use anyhow::Result;
use chrono::Utc;
use serde_json::json;

use crate::model::check::{CheckOutcome, CheckStatus};
use crate::runtime::workspace::Workspace;

pub(super) fn maybe_emit_native_help(group: &str, id: &str, err: &anyhow::Error) -> bool {
    let message = err.to_string();
    if !message.starts_with("__help__:") {
        return false;
    }
    write_line_stdout(&format!(
        "Usage: cargo run -p bijux-dna-dev -- {group} run {id} -- [args...]"
    ))
    .is_ok()
}

pub(super) fn print_check_outcome(outcome: &CheckOutcome, depth: usize) {
    let indent = "  ".repeat(depth);
    let status = match outcome.status {
        CheckStatus::Passed => "passed",
        CheckStatus::Failed => "failed",
    };
    let _ = write_line_stdout(&format!("{indent}{}: {}", outcome.id, status));
    if outcome.status == CheckStatus::Failed && !outcome.detail.trim().is_empty() {
        let _ = write_line_stdout(&format!("{indent}{}", outcome.detail.trim()));
    }
    for child in &outcome.children {
        print_check_outcome(child, depth + 1);
    }
}

pub(super) fn with_timing<F>(group: &str, command: &str, action: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    let started_at = Utc::now();
    let timer = Instant::now();
    let result = action();
    let ended_at = Utc::now();
    let exit_code = i32::from(result.is_err());
    let status = if exit_code == 0 { "ok" } else { "fail" };
    write_timing(
        group,
        command,
        status,
        exit_code,
        started_at,
        ended_at,
        timer.elapsed().as_secs(),
    );
    result
}

fn write_timing(
    group: &str,
    command: &str,
    status: &str,
    exit_code: i32,
    started_at: chrono::DateTime<Utc>,
    ended_at: chrono::DateTime<Utc>,
    duration_seconds: u64,
) {
    let Ok(workspace) = Workspace::resolve() else {
        return;
    };
    let timing_dir = std::env::var("ARTIFACT_DIR")
        .ok()
        .or_else(|| std::env::var("ISO_ROOT").ok())
        .map_or_else(
            || workspace.path("artifacts/timing"),
            |raw| {
                let path = std::path::PathBuf::from(raw);
                if path.is_absolute() {
                    path.join("timing")
                } else {
                    workspace.root.join(path).join("timing")
                }
            },
        );
    if bijux_dna_infra::ensure_dir(&timing_dir).is_err() {
        return;
    }
    let file_name = format!("{}__{}.json", group, command.replace('/', "_"));
    let payload = json!({
        "group": group,
        "command": command,
        "status": status,
        "exit_code": exit_code,
        "start_utc": started_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "end_utc": ended_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "duration_seconds": duration_seconds,
    });
    let _ = bijux_dna_infra::write_bytes(
        timing_dir.join(file_name),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        ),
    );
}

pub(super) fn write_stdout(value: &str) -> Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(value.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

pub(super) fn write_stderr(value: &str) -> Result<()> {
    let mut stderr = io::stderr().lock();
    stderr.write_all(value.as_bytes())?;
    stderr.flush()?;
    Ok(())
}

pub(super) fn write_line_stdout(value: &str) -> Result<()> {
    write_stdout(&format!("{value}\n"))
}
