//! Checked-in campaign planning for thin operator surfaces.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::plan_stage_campaign_attempts;
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
}
