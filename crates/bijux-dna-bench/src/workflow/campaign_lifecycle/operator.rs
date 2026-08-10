//! Checked-in campaign planning for thin operator surfaces.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_campaign_plan, validate_cycle_status_v2, BenchmarkCampaignPlan, BenchmarkCycleStatusV2,
};

use super::{plan_stage_campaign_attempts, verify_campaign_plan_source};
use crate::workflow::load_stage_campaign_registry;

const CAMPAIGN_REGISTRY_PATH: &str = "benchmarks/contracts/stage-campaigns.toml";
const MAX_PLAN_BYTES: u64 = 16 * 1024 * 1024;

/// Inputs for one source-locked, idempotent benchmark campaign planning pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkCyclePlanRequest {
    pub repository_root: PathBuf,
    pub run_id: String,
    pub campaign_id: String,
    pub attempt_id: String,
    pub observed_at_utc: String,
    pub output: PathBuf,
}

/// Inputs for one source-locked snapshot of persisted benchmark attempt state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkCycleStatusRequest {
    pub repository_root: PathBuf,
    pub benchmark_root: PathBuf,
    pub plan: PathBuf,
    pub observed_at_utc: String,
}

/// Plan every declared tool attempt for one permanent run and publish the immutable plan.
///
/// Repeating the request accepts byte-identical output. A different request cannot replace an
/// existing plan, so retries and changed protocols must receive new identities.
///
/// # Errors
/// Returns an error when source identity, campaign declarations, request identity, or the output
/// publication is invalid.
pub fn plan_checked_in_benchmark_campaign(
    request: &BenchmarkCyclePlanRequest,
) -> Result<bijux_dna_bench_model::BenchmarkCampaignPlan> {
    let repository_root = request
        .repository_root
        .canonicalize()
        .context("canonicalize benchmark cycle repository")?;
    let campaign_registry_path = repository_root.join(CAMPAIGN_REGISTRY_PATH);
    let repository_commit = bijux_dna_containers::api::repository_head_commit(&repository_root)
        .context("resolve benchmark cycle source commit")?;
    let campaign_registry_sha256 = bijux_dna_infra::hash_file_sha256(&campaign_registry_path)
        .context("hash benchmark campaign registry")?;
    let registry = load_stage_campaign_registry(&campaign_registry_path)?;
    let plan = plan_stage_campaign_attempts(
        &registry,
        &repository_commit,
        &campaign_registry_sha256,
        &request.run_id,
        &request.campaign_id,
        &request.attempt_id,
        &request.observed_at_utc,
    )?;
    publish_plan(&request.output, &plan)?;
    Ok(plan)
}

/// Read every persisted attempt in a source-locked campaign exactly once.
///
/// This operation does not query the scheduler, wait for files, or mutate lifecycle state. Missing
/// attempt ledgers fail closed instead of being inferred from a plan or scheduler job.
///
/// # Errors
/// Returns an error when the plan is stale or invalid, an attempt ledger is missing or invalid, or
/// the reconstructed cycle violates campaign identity and lineage rules.
pub fn status_checked_in_benchmark_campaign(
    request: &BenchmarkCycleStatusRequest,
) -> Result<BenchmarkCycleStatusV2> {
    let repository_root = request
        .repository_root
        .canonicalize()
        .context("canonicalize benchmark cycle repository")?;
    let benchmark_root = request
        .benchmark_root
        .canonicalize()
        .context("canonicalize benchmark cycle evidence root")?;
    let plan = load_checked_in_benchmark_campaign_plan(&repository_root, &request.plan)?;

    let attempts = plan
        .cycle
        .attempts
        .iter()
        .map(|attempt| {
            bijux_dna_runtime::load_attempt_lifecycle(&benchmark_root, &attempt.identity)
                .with_context(|| {
                    format!(
                        "load benchmark attempt {}/{}/{}",
                        attempt.identity.run.run_id,
                        attempt.identity.tool_id,
                        attempt.identity.attempt_id
                    )
                })
        })
        .collect::<Result<Vec<_>>>()?;
    let status = BenchmarkCycleStatusV2::new(
        plan.cycle.campaign_id,
        request.observed_at_utc.clone(),
        attempts,
    );
    validate_cycle_status_v2(&status).context("validate persisted benchmark cycle status")?;
    Ok(status)
}

/// Load a campaign plan only when it matches the exact checked-out source and registry bytes.
///
/// # Errors
/// Returns an error when the plan is unsafe, malformed, invalid, or stale for the repository.
pub fn load_checked_in_benchmark_campaign_plan(
    repository_root: &Path,
    plan_path: &Path,
) -> Result<BenchmarkCampaignPlan> {
    let repository_root =
        repository_root.canonicalize().context("canonicalize benchmark cycle repository")?;
    let plan = load_plan(plan_path)?;
    let repository_commit = bijux_dna_containers::api::repository_head_commit(&repository_root)
        .context("resolve benchmark cycle source commit")?;
    let campaign_registry_sha256 =
        bijux_dna_infra::hash_file_sha256(&repository_root.join(CAMPAIGN_REGISTRY_PATH))
            .context("hash benchmark campaign registry")?;
    verify_campaign_plan_source(&plan, &repository_commit, &campaign_registry_sha256)?;
    Ok(plan)
}

fn load_plan(path: &Path) -> Result<BenchmarkCampaignPlan> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect benchmark cycle plan {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("benchmark cycle plan is not a regular file: {}", path.display());
    }
    if metadata.len() > MAX_PLAN_BYTES {
        bail!("benchmark cycle plan exceeds the bounded size limit");
    }
    let plan: BenchmarkCampaignPlan = serde_json::from_slice(
        &fs::read(path).with_context(|| format!("read benchmark cycle plan {}", path.display()))?,
    )
    .with_context(|| format!("parse benchmark cycle plan {}", path.display()))?;
    validate_campaign_plan(&plan).context("validate benchmark cycle plan")?;
    Ok(plan)
}

fn publish_plan(output: &Path, plan: &bijux_dna_bench_model::BenchmarkCampaignPlan) -> Result<()> {
    let mut expected = serde_json::to_vec_pretty(plan).context("serialize benchmark cycle plan")?;
    expected.push(b'\n');
    match fs::symlink_metadata(output) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("benchmark cycle plan output is not a regular file: {}", output.display());
            }
            if metadata.len() > MAX_PLAN_BYTES {
                bail!("existing benchmark cycle plan exceeds the bounded size limit");
            }
            let existing = fs::read(output)
                .with_context(|| format!("read benchmark cycle plan {}", output.display()))?;
            if existing != expected {
                bail!("refusing to replace a different benchmark cycle plan: {}", output.display());
            }
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            if let Some(parent) = output.parent() {
                bijux_dna_infra::ensure_dir(parent)
                    .with_context(|| format!("create benchmark plan root {}", parent.display()))?;
            }
            bijux_dna_infra::atomic_create_bytes(output, &expected)
                .with_context(|| format!("publish benchmark cycle plan {}", output.display()))
        }
        Err(error) => {
            Err(error).with_context(|| format!("inspect benchmark cycle plan {}", output.display()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repository_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf()
    }

    #[test]
    fn checked_in_plan_is_idempotent_and_refuses_identity_replacement() -> Result<()> {
        let temporary = bijux_dna_infra::temp_dir("benchmark-cycle-plan")?;
        let output = temporary.path().join("run-2002-plan.json");
        let mut request = BenchmarkCyclePlanRequest {
            repository_root: repository_root(),
            run_id: "run-2002".to_string(),
            campaign_id: "campaign-run-2002".to_string(),
            attempt_id: "attempt-0001".to_string(),
            observed_at_utc: "2026-08-10T00:00:00Z".to_string(),
            output,
        };
        let first = plan_checked_in_benchmark_campaign(&request)?;
        let second = plan_checked_in_benchmark_campaign(&request)?;
        assert_eq!(first, second);
        assert_eq!(first.cycle.attempts.len(), 3);

        request.attempt_id = "attempt-0002".to_string();
        assert!(plan_checked_in_benchmark_campaign(&request).is_err());
        Ok(())
    }

    #[test]
    fn status_reads_each_persisted_attempt_without_scheduler_observation() -> Result<()> {
        let temporary = bijux_dna_infra::temp_dir("benchmark-cycle-status")?;
        let output = temporary.path().join("run-2002-plan.json");
        let plan = plan_checked_in_benchmark_campaign(&BenchmarkCyclePlanRequest {
            repository_root: repository_root(),
            run_id: "run-2002".to_string(),
            campaign_id: "campaign-run-2002".to_string(),
            attempt_id: "attempt-0001".to_string(),
            observed_at_utc: "2026-08-10T00:00:00Z".to_string(),
            output: output.clone(),
        })?;
        let benchmark_root = temporary.path().join("evidence");
        crate::workflow::ensure_stage_campaign_attempts(&benchmark_root, &plan)?;
        let identity = &plan.cycle.attempts[0].identity;
        bijux_dna_runtime::record_attempt_ready(&benchmark_root, identity, "2026-08-10T00:01:00Z")?;

        let status = status_checked_in_benchmark_campaign(&BenchmarkCycleStatusRequest {
            repository_root: repository_root(),
            benchmark_root,
            plan: output,
            observed_at_utc: "2026-08-10T00:02:00Z".to_string(),
        })?;
        assert_eq!(status.attempts.len(), 3);
        assert_eq!(
            status.attempts[0].current_status(),
            Some(bijux_dna_bench_model::BenchmarkLifecycleStatus::Ready)
        );
        assert!(status.attempts[0].transitions.last().is_some_and(|transition| {
            transition.jobs.is_empty() && transition.accounting.is_empty()
        }));
        Ok(())
    }
}
