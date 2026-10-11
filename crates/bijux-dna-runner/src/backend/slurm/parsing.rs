//! Owner: bijux-dna-runner
//! Fail-closed parsing for Slurm command output.

use std::collections::BTreeMap;

use anyhow::{bail, Context, Result};

use super::{SlurmAccounting, SlurmArraySpec, SlurmArrayTaskAccounting, SlurmJobState};

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

pub(super) fn parse_accounting(
    output: &str,
    job_id: &str,
    array: Option<&SlurmArraySpec>,
) -> Result<Option<SlurmAccounting>> {
    let mut matched = None;
    let mut max_rss_bytes = None;
    let mut tasks = BTreeMap::<u32, (Option<SlurmArrayTaskAccounting>, Option<u64>)>::new();
    for line in output.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let fields = line.split('|').collect::<Vec<_>>();
        if fields.len() != 8 {
            bail!("sacct row has {} fields instead of 8", fields.len());
        }
        let allocation_row = fields[0] == job_id;
        let parent_step =
            fields[0].strip_prefix(job_id).is_some_and(|suffix| suffix.starts_with('.'));
        let task_row = parse_array_row_identity(fields[0], job_id)?;
        if !allocation_row && !parent_step && task_row.is_none() {
            continue;
        }
        let row_max_rss = parse_memory_bytes(fields[5])?;
        if parent_step || allocation_row {
            max_rss_bytes = max_rss_bytes.max(row_max_rss);
        }
        if let Some((task_id, is_step)) = task_row {
            let spec = array.context("sacct returned array-task evidence for a scalar job")?;
            if task_id < spec.first_task_id || task_id > spec.last_task_id {
                bail!("sacct returned out-of-range array task {task_id} for job {job_id}");
            }
            let (task, task_max_rss) = tasks.entry(task_id).or_default();
            *task_max_rss = (*task_max_rss).max(row_max_rss);
            if is_step {
                continue;
            }
            if task.is_some() {
                bail!("sacct returned duplicate allocation rows for array task {task_id}");
            }
            *task = Some(parse_array_task(&fields, task_id)?);
            continue;
        }
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
            array_tasks: Vec::new(),
        });
    }
    let array_tasks = tasks
        .into_values()
        .map(|(task, max_rss)| {
            let mut task = task.context("sacct array task has steps but no allocation row")?;
            task.max_rss_bytes = max_rss;
            Ok(task)
        })
        .collect::<Result<Vec<_>>>()?;
    if matched.is_none() && array.is_some() && !array_tasks.is_empty() {
        matched = Some(summarize_array_accounting(job_id, &array_tasks));
    }
    if let Some(accounting) = matched.as_mut() {
        accounting.max_rss_bytes = max_rss_bytes
            .or_else(|| array_tasks.iter().filter_map(|task| task.max_rss_bytes).max());
        accounting.array_tasks = array_tasks;
        validate_terminal_array_accounting(accounting, array)?;
    }
    Ok(matched)
}

fn summarize_array_accounting(job_id: &str, tasks: &[SlurmArrayTaskAccounting]) -> SlurmAccounting {
    let nonpassing = tasks.iter().find(|task| task.state != SlurmJobState::Completed);
    let state = nonpassing.map_or(SlurmJobState::Completed, |task| task.state);
    let (exit_code, signal) = nonpassing.map_or((0, 0), |task| (task.exit_code, task.signal));
    let mut nodes = tasks
        .iter()
        .map(|task| task.node_list.as_str())
        .filter(|node| !node.is_empty())
        .collect::<Vec<_>>();
    nodes.sort_unstable();
    nodes.dedup();
    SlurmAccounting {
        job_id: job_id.to_string(),
        state,
        raw_state: "ARRAY_TASKS".to_string(),
        exit_code,
        signal,
        elapsed_seconds: tasks.iter().map(|task| task.elapsed_seconds).max().unwrap_or_default(),
        total_cpu_seconds: tasks.iter().map(|task| task.total_cpu_seconds).sum(),
        max_rss_bytes: None,
        node_list: nodes.join(","),
        reason: nonpassing.and_then(|task| task.reason.clone()),
        array_tasks: Vec::new(),
    }
}

fn parse_array_row_identity(row_job_id: &str, parent_job_id: &str) -> Result<Option<(u32, bool)>> {
    let Some(suffix) =
        row_job_id.strip_prefix(parent_job_id).and_then(|value| value.strip_prefix('_'))
    else {
        return Ok(None);
    };
    let (task, step) =
        suffix.split_once('.').map_or((suffix, None), |(task, step)| (task, Some(step)));
    if task.is_empty() || !task.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!("sacct returned a malformed array task identity: {row_job_id}");
    }
    if step.is_some_and(str::is_empty) {
        bail!("sacct returned a malformed array task step identity: {row_job_id}");
    }
    Ok(Some((task.parse().context("parse Slurm array task id")?, step.is_some())))
}

fn parse_array_task(fields: &[&str], task_id: u32) -> Result<SlurmArrayTaskAccounting> {
    let (exit_code, signal) = parse_exit_code(fields[2])?;
    Ok(SlurmArrayTaskAccounting {
        task_id,
        job_id: fields[0].to_string(),
        state: parse_state(fields[1])?,
        raw_state: fields[1].to_string(),
        exit_code,
        signal,
        elapsed_seconds: fields[3].parse().context("parse Slurm elapsed seconds")?,
        total_cpu_seconds: parse_duration_seconds(fields[4])?,
        max_rss_bytes: None,
        node_list: fields[6].to_string(),
        reason: nonempty(fields[7]),
    })
}

fn validate_terminal_array_accounting(
    accounting: &SlurmAccounting,
    array: Option<&SlurmArraySpec>,
) -> Result<()> {
    let Some(array) = array else {
        if !accounting.array_tasks.is_empty() {
            bail!("scalar Slurm accounting contains array tasks");
        }
        return Ok(());
    };
    if !accounting.state.is_terminal() {
        return Ok(());
    }
    let expected = u64::from(array.last_task_id) - u64::from(array.first_task_id) + 1;
    if accounting.array_tasks.len() as u64 != expected {
        bail!(
            "terminal Slurm array accounting has {} tasks instead of {expected}",
            accounting.array_tasks.len()
        );
    }
    if accounting.array_tasks.iter().any(|task| !task.state.is_terminal()) {
        bail!("terminal Slurm array accounting contains a non-terminal task");
    }
    Ok(())
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
    let whole_seconds = match seconds.split_once('.') {
        Some((whole, fraction)) => {
            if fraction.is_empty() || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
                bail!("Slurm duration contains a malformed fractional second");
            }
            whole
        }
        None => seconds,
    };
    let seconds: u64 = whole_seconds.parse().context("parse Slurm duration seconds")?;
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
