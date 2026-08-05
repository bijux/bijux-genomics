//! Owner: bijux-dna-bench
//! Crash-recoverable publication of a complete planned campaign cycle.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_campaign_plan, validate_execution_spec, BenchmarkAttemptRecordV2,
    BenchmarkCampaignPlan, BenchmarkExecutionSpec, BenchmarkLifecycleStatus,
};
use bijux_dna_runtime::{
    benchmark_attempt_directory, ensure_attempt_lifecycle, load_attempt_lifecycle,
};

/// Persist every planned campaign attempt under its immutable run/tool/campaign/attempt identity.
///
/// Existing attempts with the same immutable planned origin are accepted after later lifecycle
/// transitions. A complete preflight rejects known conflicts before any missing attempt is created.
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
        let directory = benchmark_attempt_directory(benchmark_root, &attempt.identity)?;
        if is_planned_origin(attempt) {
            if directory.exists() {
                validate_attempt_origin(benchmark_root, attempt, &directory)?;
            }
        } else if is_terminal_history(attempt) {
            if !directory.exists() {
                bail!(
                    "benchmark retry history is absent from persistent storage: {}",
                    directory.display()
                );
            }
            let existing = load_attempt_lifecycle(benchmark_root, &attempt.identity)?;
            if existing != **attempt {
                bail!(
                    "benchmark retry history differs from persistent evidence: {}",
                    directory.display()
                );
            }
        } else {
            bail!("benchmark campaign plan contains unsupported active history");
        }
    }

    attempts
        .into_iter()
        .filter(|attempt| is_planned_origin(attempt))
        .map(|attempt| {
            let directory = benchmark_attempt_directory(benchmark_root, &attempt.identity)?;
            if directory.exists() {
                validate_attempt_origin(benchmark_root, attempt, &directory)?;
                Ok(directory)
            } else {
                ensure_attempt_lifecycle(benchmark_root, attempt)
                    .context("ensure benchmark campaign attempt")
            }
        })
        .collect()
}

fn is_planned_origin(attempt: &BenchmarkAttemptRecordV2) -> bool {
    attempt.transitions.len() == 1
        && attempt.current_status() == Some(BenchmarkLifecycleStatus::Planned)
}

fn is_terminal_history(attempt: &BenchmarkAttemptRecordV2) -> bool {
    matches!(
        attempt.current_status(),
        Some(
            BenchmarkLifecycleStatus::Passed
                | BenchmarkLifecycleStatus::Failed
                | BenchmarkLifecycleStatus::Superseded
                | BenchmarkLifecycleStatus::BlockedExternal
        )
    )
}

fn validate_attempt_origin(
    benchmark_root: &Path,
    planned: &BenchmarkAttemptRecordV2,
    directory: &Path,
) -> Result<()> {
    let existing = load_attempt_lifecycle(benchmark_root, &planned.identity)?;
    if existing.identity != planned.identity
        || existing.supersedes_attempt_id != planned.supersedes_attempt_id
        || existing.transitions.first() != planned.transitions.first()
    {
        bail!(
            "benchmark campaign attempt conflicts with immutable origin evidence: {}",
            directory.display()
        );
    }
    Ok(())
}

/// Load and validate one persisted execution specification for an initialized attempt.
///
/// # Errors
/// Returns an error when the attempt or specification is missing, malformed, symlinked, or bound
/// to a different immutable result identity.
pub fn load_execution_spec(
    benchmark_root: &Path,
    identity: &bijux_dna_bench_model::BenchmarkAttemptIdentity,
) -> Result<BenchmarkExecutionSpec> {
    let directory = benchmark_attempt_directory(benchmark_root, identity)?;
    load_attempt_lifecycle(benchmark_root, identity).context("load execution attempt lifecycle")?;
    let path = directory.join("execution-spec.json");
    let metadata = fs::symlink_metadata(&path)
        .with_context(|| format!("inspect benchmark execution spec {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("benchmark execution specification is not a regular file");
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(&path, 16 * 1024 * 1024)
        .with_context(|| format!("read benchmark execution spec {}", path.display()))?;
    let spec: BenchmarkExecutionSpec = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse benchmark execution spec {}", path.display()))?;
    validate_execution_spec(&spec).context("validate persisted execution specification")?;
    if spec.identity != *identity || Path::new(&spec.outputs.result_root) != directory {
        bail!("benchmark execution specification differs from immutable attempt identity");
    }
    Ok(spec)
}

/// Persist one execution specification beside its initialized attempt lifecycle.
///
/// Existing structurally identical content is accepted for crash recovery. The result root and
/// planned attempt evidence must already match the same immutable identity.
///
/// # Errors
/// Returns an error when the specification is invalid, its attempt is absent or no longer wholly
/// planned, its result root differs from runtime path policy, or existing content conflicts.
pub fn ensure_execution_spec(
    benchmark_root: &Path,
    spec: &BenchmarkExecutionSpec,
) -> Result<PathBuf> {
    validate_execution_spec(spec).context("validate benchmark execution specification")?;
    let directory = benchmark_attempt_directory(benchmark_root, &spec.identity)?;
    if directory != Path::new(&spec.outputs.result_root) {
        bail!("benchmark execution result root differs from immutable attempt path");
    }
    let attempt = load_attempt_lifecycle(benchmark_root, &spec.identity)
        .context("load initialized benchmark attempt")?;
    if attempt.transitions.len() != 1
        || attempt.current_status() != Some(BenchmarkLifecycleStatus::Planned)
    {
        bail!("benchmark execution specification requires a wholly planned attempt");
    }
    let path = directory.join("execution-spec.json");
    match fs::symlink_metadata(&path) {
        Ok(metadata) => {
            if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
                bail!("benchmark execution specification is not a regular file");
            }
            let bytes = bijux_dna_infra::read_to_end_bounded(&path, 16 * 1024 * 1024)
                .with_context(|| format!("read benchmark execution spec {}", path.display()))?;
            let existing: BenchmarkExecutionSpec = serde_json::from_slice(&bytes)
                .with_context(|| format!("parse benchmark execution spec {}", path.display()))?;
            validate_execution_spec(&existing)
                .context("validate existing execution specification")?;
            if existing != *spec {
                bail!(
                    "benchmark execution specification conflicts with immutable attempt evidence"
                );
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_json(&path, spec)
                .with_context(|| format!("write benchmark execution spec {}", path.display()))?;
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("inspect benchmark execution spec {}", path.display()));
        }
    }
    Ok(path)
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
