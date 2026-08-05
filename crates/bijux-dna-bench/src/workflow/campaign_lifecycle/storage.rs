//! Owner: bijux-dna-bench
//! Crash-recoverable publication of a complete planned campaign cycle.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_campaign_plan, BenchmarkAttemptRecordV2, BenchmarkCampaignPlan,
    BenchmarkLifecycleStatus,
};
use bijux_dna_runtime::{
    benchmark_attempt_directory, ensure_attempt_lifecycle, load_attempt_lifecycle,
};

/// Persist every planned campaign attempt under its immutable run/tool/campaign/attempt identity.
///
/// Existing structurally identical planned records are accepted for crash recovery. A complete
/// preflight rejects known conflicts before any missing attempt is created.
///
/// # Errors
/// Returns an error when the cycle is invalid or not wholly planned, an existing identity differs,
/// or runtime cannot atomically initialize an attempt.
pub fn ensure_stage_campaign_attempts(
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
) -> Result<Vec<PathBuf>> {
    validate_campaign_plan(plan).context("validate benchmark campaign plan")?;
    let mut attempts = plan.cycle.attempts.iter().collect::<Vec<_>>();
    attempts.sort_by(|left, right| attempt_sort_key(left).cmp(&attempt_sort_key(right)));
    for attempt in &attempts {
        if attempt.transitions.len() != 1
            || attempt.current_status() != Some(BenchmarkLifecycleStatus::Planned)
        {
            bail!("benchmark campaign initialization requires wholly planned attempts");
        }
        let directory = benchmark_attempt_directory(benchmark_root, &attempt.identity)?;
        if directory.exists()
            && load_attempt_lifecycle(benchmark_root, &attempt.identity)? != **attempt
        {
            bail!(
                "benchmark campaign attempt conflicts with immutable evidence: {}",
                directory.display()
            );
        }
    }

    attempts
        .into_iter()
        .map(|attempt| {
            ensure_attempt_lifecycle(benchmark_root, attempt)
                .context("ensure benchmark campaign attempt")
        })
        .collect()
}

fn attempt_sort_key(attempt: &BenchmarkAttemptRecordV2) -> (&str, &str, &str, &str, &str) {
    (
        &attempt.identity.run.run_id,
        &attempt.identity.run.stage_id,
        &attempt.identity.tool_id,
        &attempt.identity.campaign_id,
        &attempt.identity.attempt_id,
    )
}
