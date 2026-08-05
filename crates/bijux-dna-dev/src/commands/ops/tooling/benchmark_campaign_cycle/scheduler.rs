//! Slurm evidence translation for benchmark campaign operations.

use super::Result;
use anyhow::bail;

pub(super) fn benchmark_accounting(
    accounting: &bijux_dna_runner::api::SlurmAccounting,
) -> Result<bijux_dna_bench::BenchmarkSchedulerAccounting> {
    let state = match accounting.state {
        bijux_dna_runner::api::SlurmJobState::Completed => {
            bijux_dna_bench::BenchmarkSchedulerTerminalState::Completed
        }
        bijux_dna_runner::api::SlurmJobState::Failed => {
            bijux_dna_bench::BenchmarkSchedulerTerminalState::Failed
        }
        bijux_dna_runner::api::SlurmJobState::Cancelled => {
            bijux_dna_bench::BenchmarkSchedulerTerminalState::Cancelled
        }
        bijux_dna_runner::api::SlurmJobState::Timeout => {
            bijux_dna_bench::BenchmarkSchedulerTerminalState::Timeout
        }
        bijux_dna_runner::api::SlurmJobState::Preempted => {
            bijux_dna_bench::BenchmarkSchedulerTerminalState::Preempted
        }
        bijux_dna_runner::api::SlurmJobState::NodeFailure => {
            bijux_dna_bench::BenchmarkSchedulerTerminalState::NodeFailure
        }
        bijux_dna_runner::api::SlurmJobState::OutOfMemory => {
            bijux_dna_bench::BenchmarkSchedulerTerminalState::OutOfMemory
        }
        _ => bail!("benchmark accounting is not terminal"),
    };
    Ok(bijux_dna_bench::BenchmarkSchedulerAccounting {
        scheduler: "slurm".to_string(),
        job_id: accounting.job_id.clone(),
        state,
        raw_state: accounting.raw_state.clone(),
        exit_code: accounting.exit_code,
        signal: accounting.signal,
        elapsed_seconds: accounting.elapsed_seconds,
        total_cpu_seconds: accounting.total_cpu_seconds,
        max_rss_bytes: accounting.max_rss_bytes,
        node_list: accounting.node_list.clone(),
        reason: accounting.reason.clone(),
    })
}

pub(super) fn slurm_state(state: bijux_dna_runner::api::SlurmJobState) -> &'static str {
    match state {
        bijux_dna_runner::api::SlurmJobState::Submitted => "submitted",
        bijux_dna_runner::api::SlurmJobState::Pending => "pending",
        bijux_dna_runner::api::SlurmJobState::Running => "running",
        bijux_dna_runner::api::SlurmJobState::Completed => "completed",
        bijux_dna_runner::api::SlurmJobState::Failed => "failed",
        bijux_dna_runner::api::SlurmJobState::Cancelled => "cancelled",
        bijux_dna_runner::api::SlurmJobState::Timeout => "timeout",
        bijux_dna_runner::api::SlurmJobState::Preempted => "preempted",
        bijux_dna_runner::api::SlurmJobState::NodeFailure => "node_failure",
        bijux_dna_runner::api::SlurmJobState::OutOfMemory => "out_of_memory",
        bijux_dna_runner::api::SlurmJobState::AccountingPending => "accounting_pending",
    }
}

pub(super) fn lifecycle_status(status: bijux_dna_bench::BenchmarkLifecycleStatus) -> &'static str {
    match status {
        bijux_dna_bench::BenchmarkLifecycleStatus::Planned => "planned",
        bijux_dna_bench::BenchmarkLifecycleStatus::Ready => "ready",
        bijux_dna_bench::BenchmarkLifecycleStatus::Submitted => "submitted",
        bijux_dna_bench::BenchmarkLifecycleStatus::Running => "running",
        bijux_dna_bench::BenchmarkLifecycleStatus::Collecting => "collecting",
        bijux_dna_bench::BenchmarkLifecycleStatus::Passed => "passed",
        bijux_dna_bench::BenchmarkLifecycleStatus::Failed => "failed",
        bijux_dna_bench::BenchmarkLifecycleStatus::Superseded => "superseded",
        bijux_dna_bench::BenchmarkLifecycleStatus::BlockedExternal => "blocked_external",
    }
}
