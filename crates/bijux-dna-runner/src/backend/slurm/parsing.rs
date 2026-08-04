//! Owner: bijux-dna-runner
//! Fail-closed parsing for Slurm command output.

use anyhow::{bail, Context, Result};

use super::{SlurmAccounting, SlurmJobState};

pub(super) fn parse_job_id(output: &str) -> Result<String> {
    let job_id = output.trim().split_once(';').map_or(output.trim(), |(id, _)| id);
    if job_id.is_empty() || !job_id.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!("sbatch returned a malformed Slurm job id: {output:?}");
    }
    Ok(job_id.to_string())
}

pub(super) fn parse_queue_state(output: &str) -> Result<Option<SlurmJobState>> {
    let states = output.lines().map(str::trim).filter(|line| !line.is_empty()).collect::<Vec<_>>();
    if states.is_empty() {
        return Ok(None);
    }
    if states.len() != 1 {
        bail!("squeue returned multiple allocation states");
    }
    Ok(Some(parse_state(states[0])?))
}

pub(super) fn parse_accounting(output: &str, job_id: &str) -> Result<Option<SlurmAccounting>> {
    let mut matched = None;
    let mut max_rss_bytes = None;
    for line in output.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let fields = line.split('|').collect::<Vec<_>>();
        if fields.len() != 8 {
            bail!("sacct row has {} fields instead of 8", fields.len());
        }
        let allocation_row = fields[0] == job_id;
        let step_row = fields[0].strip_prefix(job_id).is_some_and(|suffix| suffix.starts_with('.'));
        if !allocation_row && !step_row {
            continue;
        }
        max_rss_bytes = max_rss_bytes.max(parse_memory_bytes(fields[5])?);
        if !allocation_row {
            continue;
        }
        if matched.is_some() {
            bail!("sacct returned duplicate allocation rows for job {job_id}");
        }
        let (exit_code, signal) = parse_exit_code(fields[2])?;
        matched = Some(SlurmAccounting {
            job_id: job_id.to_string(),
            state: parse_state(fields[1])?,
            raw_state: fields[1].to_string(),
            exit_code,
            signal,
            elapsed_seconds: fields[3].parse().context("parse Slurm elapsed seconds")?,
            total_cpu_seconds: parse_duration_seconds(fields[4])?,
            max_rss_bytes: None,
            node_list: fields[6].to_string(),
            reason: nonempty(fields[7]),
        });
    }
    if let Some(accounting) = matched.as_mut() {
        accounting.max_rss_bytes = max_rss_bytes;
    }
    Ok(matched)
}

fn parse_state(raw: &str) -> Result<SlurmJobState> {
    let normalized = raw.trim_end_matches('+').split_whitespace().next().unwrap_or_default();
    match normalized {
        "SUBMITTED" => Ok(SlurmJobState::Submitted),
        "PENDING" | "CONFIGURING" | "REQUEUED" | "RESIZING" => Ok(SlurmJobState::Pending),
        "RUNNING" | "COMPLETING" => Ok(SlurmJobState::Running),
        "COMPLETED" => Ok(SlurmJobState::Completed),
        "FAILED" | "BOOT_FAIL" | "DEADLINE" | "REVOKED" => Ok(SlurmJobState::Failed),
        "CANCELLED" => Ok(SlurmJobState::Cancelled),
        "TIMEOUT" => Ok(SlurmJobState::Timeout),
        "PREEMPTED" => Ok(SlurmJobState::Preempted),
        "NODE_FAIL" => Ok(SlurmJobState::NodeFailure),
        "OUT_OF_MEMORY" => Ok(SlurmJobState::OutOfMemory),
        _ => bail!("unsupported Slurm job state: {raw}"),
    }
}

fn parse_exit_code(raw: &str) -> Result<(i32, i32)> {
    let (code, signal) = raw
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("Slurm exit code must use code:signal"))?;
    Ok((code.parse().context("parse Slurm exit code")?, signal.parse().context("parse signal")?))
}

fn parse_duration_seconds(raw: &str) -> Result<u64> {
    let (days, clock) =
        raw.split_once('-').map_or((None, raw), |(days, clock)| (Some(days), clock));
    let days =
        days.map_or(Ok(0), |value| value.parse::<u64>().context("parse Slurm duration days"))?;
    let parts = clock.split(':').collect::<Vec<_>>();
    let (hours, minutes, seconds) = match parts.as_slice() {
        [hours, minutes, seconds] => (*hours, *minutes, *seconds),
        [minutes, seconds] => ("0", *minutes, *seconds),
        _ => bail!("Slurm duration must use [[days-]HH:]MM:SS"),
    };
    let hours: u64 = hours.parse().context("parse Slurm duration hours")?;
    let minutes: u64 = minutes.parse().context("parse Slurm duration minutes")?;
    let seconds: u64 = seconds.parse().context("parse Slurm duration seconds")?;
    if minutes >= 60 || seconds >= 60 {
        bail!("Slurm duration contains an out-of-range component");
    }
    let clock_seconds = hours
        .checked_mul(3600)
        .and_then(|value| minutes.checked_mul(60).and_then(|minutes| value.checked_add(minutes)))
        .and_then(|value| value.checked_add(seconds))
        .ok_or_else(|| anyhow::anyhow!("Slurm duration overflow"))?;
    days.checked_mul(86_400)
        .and_then(|value| value.checked_add(clock_seconds))
        .ok_or_else(|| anyhow::anyhow!("Slurm duration overflow"))
}

fn parse_memory_bytes(raw: &str) -> Result<Option<u64>> {
    if raw.is_empty() || raw == "Unknown" {
        return Ok(None);
    }
    let (number, multiplier) = match raw.as_bytes().last().copied() {
        Some(b'K') => (&raw[..raw.len() - 1], 1024_u64),
        Some(b'M') => (&raw[..raw.len() - 1], 1024_u64.pow(2)),
        Some(b'G') => (&raw[..raw.len() - 1], 1024_u64.pow(3)),
        Some(b'T') => (&raw[..raw.len() - 1], 1024_u64.pow(4)),
        Some(byte) if byte.is_ascii_digit() => (raw, 1),
        _ => bail!("Slurm MaxRSS has an unsupported unit: {raw}"),
    };
    let value: u64 = number.parse().context("parse Slurm MaxRSS")?;
    Ok(Some(value.checked_mul(multiplier).ok_or_else(|| anyhow::anyhow!("Slurm MaxRSS overflow"))?))
}

fn nonempty(value: &str) -> Option<String> {
    (!value.trim().is_empty() && value != "None").then(|| value.to_string())
}
