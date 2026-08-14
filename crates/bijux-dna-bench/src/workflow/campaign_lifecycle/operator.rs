//! Owner: bijux-dna-bench
//! Checked-in campaign planning for thin operator surfaces.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_campaign_plan, validate_cycle_admission, validate_cycle_status_v2,
    validate_utc_timestamp, BenchmarkAttemptIdentity, BenchmarkCampaignPlan,
    BenchmarkCycleAdmission, BenchmarkCycleAdmissionState, BenchmarkCycleAdmissionUnit,
    BenchmarkCycleStatusV2, BenchmarkEvidenceReference, BenchmarkLifecycleStatus,
};

use super::{
    plan_benchmark_case_attempts, plan_benchmark_case_retry_attempts, plan_stage_campaign_attempts,
    verify_campaign_plan_source, verify_case_campaign_plan_source,
};
use crate::workflow::{
    load_benchmark_case_matrix, load_stage_campaign_registry,
    load_stage_interface_catalog_from_repo,
};

const CAMPAIGN_REGISTRY_PATH: &str = "benchmarks/contracts/stage-campaigns.toml";
const STAGE_INTERFACE_REGISTRY_PATH: &str = "benchmarks/contracts/stage-interfaces.toml";
const MAX_PLAN_BYTES: u64 = 16 * 1024 * 1024;

/// Inputs for one source-locked, idempotent benchmark campaign planning pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkCyclePlanRequest {
    pub repository_root: PathBuf,
    pub run_id: String,
    pub case_matrix: Option<PathBuf>,
    pub campaign_id: String,
    pub attempt_id: String,
    pub observed_at_utc: String,
    pub output: PathBuf,
}

/// Inputs for one source-descendant retry of an exact case-scoped benchmark campaign.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkCycleRetryRequest {
    pub repository_root: PathBuf,
    pub benchmark_root: PathBuf,
    pub case_matrix: PathBuf,
    pub prior_plan: PathBuf,
    pub prior_case_matrix: PathBuf,
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
    pub case_matrix: Option<PathBuf>,
    pub observed_at_utc: String,
}

/// Inputs for one independent-unit benchmark output admission pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkCycleAdmissionRequest {
    pub repository_root: PathBuf,
    pub study_root: PathBuf,
    pub benchmark_root: PathBuf,
    pub plan: PathBuf,
    pub case_matrix: Option<PathBuf>,
    pub admitted_at_utc: String,
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
    let repository_commit =
        bijux_dna_containers::api::clean_repository_head_commit(&repository_root)
            .context("require a clean benchmark cycle source repository")?;
    let campaign_registry_sha256 = bijux_dna_infra::hash_file_sha256(&campaign_registry_path)
        .context("hash benchmark campaign registry")?;
    let plan = if let Some(case_matrix_path) = &request.case_matrix {
        let matrix = load_benchmark_case_matrix(case_matrix_path)?;
        if matrix.repository_commit != repository_commit
            || matrix.campaign_registry_sha256 != campaign_registry_sha256
            || matrix.cases.iter().any(|case| case.run_id != request.run_id)
        {
            bail!("benchmark case matrix differs from the requested source-locked run");
        }
        plan_benchmark_case_attempts(
            &matrix,
            &request.campaign_id,
            &request.attempt_id,
            &request.observed_at_utc,
        )?
    } else {
        let registry = load_stage_campaign_registry(&campaign_registry_path)?;
        plan_stage_campaign_attempts(
            &registry,
            &repository_commit,
            &campaign_registry_sha256,
            &request.run_id,
            &request.campaign_id,
            &request.attempt_id,
            &request.observed_at_utc,
        )?
    };
    publish_plan(&request.output, &plan)?;
    Ok(plan)
}

/// Plan one explicit retry for every terminal lineage tip in a case-scoped campaign.
///
/// The prior plan and its matrix remain immutable authorities. Their source must be an ancestor of
/// the clean current repository, while the new matrix must match the current source exactly.
/// Persisted terminal attempts provide the retry lineage and cannot be replaced by plan snapshots.
///
/// # Errors
/// Returns an error when source ancestry, current or prior matrices, persisted terminal lineage,
/// retry identity, timestamp, or immutable output publication is invalid.
pub fn retry_checked_in_benchmark_campaign(
    request: &BenchmarkCycleRetryRequest,
) -> Result<BenchmarkCampaignPlan> {
    validate_utc_timestamp(&request.observed_at_utc, "benchmark retry observed_at_utc")
        .context("validate benchmark retry timestamp")?;
    let repository_root = request
        .repository_root
        .canonicalize()
        .context("canonicalize benchmark retry repository")?;
    let benchmark_root = request
        .benchmark_root
        .canonicalize()
        .context("canonicalize benchmark retry evidence root")?;
    let repository_commit =
        bijux_dna_containers::api::clean_repository_head_commit(&repository_root)
            .context("require a clean benchmark retry source repository")?;
    let campaign_registry_sha256 =
        bijux_dna_infra::hash_file_sha256(&repository_root.join(CAMPAIGN_REGISTRY_PATH))
            .context("hash benchmark retry campaign registry")?;

    let matrix = load_benchmark_case_matrix(&request.case_matrix)?;
    if matrix.repository_commit != repository_commit
        || matrix.campaign_registry_sha256 != campaign_registry_sha256
    {
        bail!("benchmark retry case matrix differs from the current source");
    }

    let prior_plan = load_plan(&request.prior_plan)?;
    let prior_matrix = load_benchmark_case_matrix(&request.prior_case_matrix)?;
    verify_case_campaign_plan_source(&prior_plan, &prior_matrix)
        .context("verify prior benchmark retry authorities")?;
    if !bijux_dna_containers::api::repository_commit_is_ancestor_of_head(
        &repository_root,
        &prior_plan.repository_commit,
    )? {
        bail!("benchmark retry prior source is not an ancestor of the current source");
    }

    let prior_attempts = prior_plan
        .cycle
        .attempts
        .iter()
        .map(|attempt| {
            bijux_dna_runtime::load_attempt_lifecycle(&benchmark_root, &attempt.identity)
                .with_context(|| {
                    format!(
                        "load prior benchmark retry attempt {}/{}/{}",
                        attempt.identity.run.run_id,
                        attempt.identity.tool_id,
                        attempt.identity.attempt_id
                    )
                })
        })
        .collect::<Result<Vec<_>>>()?;
    let retry = plan_benchmark_case_retry_attempts(
        &matrix,
        prior_attempts,
        &request.attempt_id,
        &request.observed_at_utc,
    )?;
    publish_plan(&request.output, &retry)?;
    Ok(retry)
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
    let plan = load_checked_in_benchmark_campaign_plan(
        &repository_root,
        &request.plan,
        request.case_matrix.as_deref(),
    )?;

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

/// Verify and admit every independently ready output closure in one campaign.
///
/// Non-ready attempts are reported without mutation. A failure for one attempt is retained in its
/// unit outcome while all other independent ready attempts continue through the same pass.
///
/// # Errors
/// Returns an error only when top-level source, plan, timestamp, or evidence-root authorities are
/// invalid. Attempt-local admission failures are returned in [`BenchmarkCycleAdmissionUnit`].
pub fn admit_checked_in_benchmark_campaign(
    request: &BenchmarkCycleAdmissionRequest,
) -> Result<BenchmarkCycleAdmission> {
    validate_utc_timestamp(&request.admitted_at_utc, "benchmark admission admitted_at_utc")
        .context("validate benchmark admission timestamp")?;
    let repository_root = request
        .repository_root
        .canonicalize()
        .context("canonicalize benchmark admission repository")?;
    let study_root =
        request.study_root.canonicalize().context("canonicalize benchmark admission STUDY root")?;
    let benchmark_root = request
        .benchmark_root
        .canonicalize()
        .context("canonicalize benchmark admission evidence root")?;
    let plan = load_checked_in_benchmark_campaign_plan(
        &repository_root,
        &request.plan,
        request.case_matrix.as_deref(),
    )?;
    let mut attempts = plan
        .cycle
        .attempts
        .iter()
        .filter(|attempt| {
            attempt.transitions.len() == 1
                && attempt.current_status() == Some(BenchmarkLifecycleStatus::Planned)
        })
        .collect::<Vec<_>>();
    attempts.sort_by(|left, right| {
        (left.identity.case_id.as_deref(), left.identity.tool_id.as_str())
            .cmp(&(right.identity.case_id.as_deref(), right.identity.tool_id.as_str()))
    });

    let units = attempts
        .into_iter()
        .map(|attempt| {
            admit_attempt(&study_root, &benchmark_root, &attempt.identity, &request.admitted_at_utc)
        })
        .collect();
    let admission = BenchmarkCycleAdmission::new(
        plan.cycle.campaign_id,
        request.admitted_at_utc.clone(),
        units,
    );
    validate_cycle_admission(&admission)?;
    Ok(admission)
}

fn admit_attempt(
    study_root: &Path,
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    admitted_at_utc: &str,
) -> BenchmarkCycleAdmissionUnit {
    let lifecycle = match bijux_dna_runtime::load_attempt_lifecycle(benchmark_root, identity) {
        Ok(lifecycle) => lifecycle,
        Err(error) => return admission_failure(benchmark_root, identity, None, error),
    };
    let prior_status = lifecycle.current_status();
    if !matches!(
        prior_status,
        Some(BenchmarkLifecycleStatus::Collecting | BenchmarkLifecycleStatus::Passed)
    ) {
        return BenchmarkCycleAdmissionUnit {
            identity: identity.clone(),
            prior_status,
            current_status: prior_status,
            state: BenchmarkCycleAdmissionState::NotReady,
            evidence: None,
            error: None,
        };
    }
    let result = (|| -> Result<BenchmarkEvidenceReference> {
        let spec = super::load_execution_spec(benchmark_root, identity)?;
        let publication = if prior_status == Some(BenchmarkLifecycleStatus::Passed) {
            crate::revalidate_benchmark_execution_outputs(study_root, &spec)?
        } else {
            crate::admit_benchmark_execution_outputs(study_root, &spec, admitted_at_utc)?
        };
        bijux_dna_runtime::record_attempt_acceptance(
            benchmark_root,
            identity,
            &publication.report.admitted_at_utc,
            vec![publication.evidence.clone()],
        )?;
        Ok(publication.evidence)
    })();
    match result {
        Ok(evidence) => BenchmarkCycleAdmissionUnit {
            identity: identity.clone(),
            prior_status,
            current_status: Some(BenchmarkLifecycleStatus::Passed),
            state: if prior_status == Some(BenchmarkLifecycleStatus::Passed) {
                BenchmarkCycleAdmissionState::AlreadyAdmitted
            } else {
                BenchmarkCycleAdmissionState::Admitted
            },
            evidence: Some(evidence),
            error: None,
        },
        Err(error) => admission_failure(benchmark_root, identity, prior_status, error),
    }
}

fn admission_failure(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    prior_status: Option<BenchmarkLifecycleStatus>,
    error: anyhow::Error,
) -> BenchmarkCycleAdmissionUnit {
    let current_status = bijux_dna_runtime::load_attempt_lifecycle(benchmark_root, identity)
        .ok()
        .and_then(|lifecycle| lifecycle.current_status());
    BenchmarkCycleAdmissionUnit {
        identity: identity.clone(),
        prior_status,
        current_status: current_status.or(prior_status),
        state: BenchmarkCycleAdmissionState::Failed,
        evidence: None,
        error: Some(format!("{error:#}")),
    }
}

/// Load a campaign plan only when it matches the exact checked-out source, registry bytes, and
/// case matrix when the plan is case-scoped.
///
/// # Errors
/// Returns an error when the plan is unsafe, malformed, invalid, or stale for the repository.
pub fn load_checked_in_benchmark_campaign_plan(
    repository_root: &Path,
    plan_path: &Path,
    case_matrix_path: Option<&Path>,
) -> Result<BenchmarkCampaignPlan> {
    let repository_root =
        repository_root.canonicalize().context("canonicalize benchmark cycle repository")?;
    let plan = load_plan(plan_path)?;
    let repository_commit =
        bijux_dna_containers::api::clean_repository_head_commit(&repository_root)
            .context("require a clean benchmark cycle source repository")?;
    let campaign_registry_sha256 =
        bijux_dna_infra::hash_file_sha256(&repository_root.join(CAMPAIGN_REGISTRY_PATH))
            .context("hash benchmark campaign registry")?;
    match (plan.case_matrix_sha256.as_ref(), case_matrix_path) {
        (Some(_), Some(path)) => {
            let matrix = load_benchmark_case_matrix(path)?;
            if matrix.repository_commit != repository_commit
                || matrix.campaign_registry_sha256 != campaign_registry_sha256
            {
                bail!("benchmark case matrix source identity is stale");
            }
            verify_case_campaign_plan_source(&plan, &matrix)?;
            let registry =
                load_stage_campaign_registry(&repository_root.join(CAMPAIGN_REGISTRY_PATH))?;
            let interface_path = repository_root.join(STAGE_INTERFACE_REGISTRY_PATH);
            let interfaces =
                load_stage_interface_catalog_from_repo(&interface_path, &repository_root)?;
            let bindings = bijux_dna_stage_contract::canonical_tool_bindings()?;
            let binding_bytes =
                serde_json::to_vec(&bindings.bindings().cloned().collect::<Vec<_>>())?;
            super::authorities::validate_case_source_authorities(
                &matrix,
                &plan,
                &registry,
                &interfaces,
                &bindings,
                &repository_commit,
                &campaign_registry_sha256,
                &bijux_dna_infra::hash_file_sha256(&interface_path)?,
                &bijux_dna_infra::hash_bytes_sha256(&binding_bytes),
            )?;
        }
        (Some(_), None) => {
            bail!("case-scoped benchmark plan requires its case-matrix authority");
        }
        (None, Some(_)) => {
            bail!("legacy benchmark plan cannot be paired with a case-matrix authority");
        }
        (None, None) => {
            verify_campaign_plan_source(&plan, &repository_commit, &campaign_registry_sha256)?;
        }
    }
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
    use std::process::Command;

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf()
    }

    fn clean_repository() -> Result<tempfile::TempDir> {
        let repository = bijux_dna_infra::temp_dir("benchmark-cycle-repository")?;
        let contracts = repository.path().join("benchmarks/contracts");
        fs::create_dir_all(&contracts)?;
        fs::copy(
            workspace_root().join(CAMPAIGN_REGISTRY_PATH),
            contracts.join("stage-campaigns.toml"),
        )?;
        for arguments in [
            vec!["init", "--initial-branch=main"],
            vec!["config", "user.email", "tests@bijux.invalid"],
            vec!["config", "user.name", "Bijux Tests"],
            vec!["add", CAMPAIGN_REGISTRY_PATH],
            vec!["commit", "-m", "test: seed campaign registry"],
        ] {
            let output = Command::new("git")
                .current_dir(repository.path())
                .args(arguments)
                .output()
                .context("execute benchmark cycle Git fixture command")?;
            if !output.status.success() {
                bail!(
                    "benchmark cycle Git fixture command failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                );
            }
        }
        Ok(repository)
    }

    fn case_matrix_for(
        repository: &Path,
        repository_commit: &str,
    ) -> Result<crate::workflow::BenchmarkCaseMatrixV2> {
        let (mut campaigns, _, mut interfaces) =
            crate::workflow::load_checked_in_benchmark_planning_contracts()?;
        campaigns.campaigns.retain(|campaign| campaign.run_id == "run-2042");
        interfaces
            .interfaces
            .retain(|interface| interface.stage_id == "fastq.profile_read_lengths");
        let campaign = campaigns.campaigns.first().context("run-2042 campaign")?;
        let data_snapshots = [
            (&campaign.corpus_catalog_ids, crate::workflow::BenchmarkDataSnapshotRoleV1::Corpus),
            (
                &campaign.reference_catalog_ids,
                crate::workflow::BenchmarkDataSnapshotRoleV1::Reference,
            ),
            (
                &campaign.database_catalog_ids,
                crate::workflow::BenchmarkDataSnapshotRoleV1::Database,
            ),
        ]
        .into_iter()
        .flat_map(|(ids, role)| {
            ids.iter().map(move |id| crate::workflow::BenchmarkDataSnapshotV1 {
                catalog_id: id.clone(),
                role,
                catalog_sha256: "8".repeat(64),
                sample_ids: if role == crate::workflow::BenchmarkDataSnapshotRoleV1::Corpus {
                    vec!["sample-a".to_string()]
                } else {
                    Vec::new()
                },
            })
        })
        .collect::<Vec<_>>();
        let bindings = bijux_dna_stage_contract::canonical_tool_bindings()?;
        let binding_bytes = serde_json::to_vec(&bindings.bindings().cloned().collect::<Vec<_>>())?;
        crate::workflow::plan_benchmark_case_matrix(&crate::workflow::BenchmarkCaseMatrixRequest {
            repository_commit,
            campaign_registry_sha256: &bijux_dna_infra::hash_file_sha256(
                &repository.join(CAMPAIGN_REGISTRY_PATH),
            )?,
            interface_catalog_sha256: &"9".repeat(64),
            tool_binding_registry_sha256: &bijux_dna_infra::hash_bytes_sha256(&binding_bytes),
            campaigns: &campaigns,
            interfaces: &interfaces,
            bindings: &bindings,
            data_snapshots: &data_snapshots,
        })
    }

    #[test]
    fn checked_in_plan_is_idempotent_and_refuses_identity_replacement() -> Result<()> {
        let repository = clean_repository()?;
        let temporary = bijux_dna_infra::temp_dir("benchmark-cycle-plan")?;
        let output = temporary.path().join("run-2002-plan.json");
        let mut request = BenchmarkCyclePlanRequest {
            repository_root: repository.path().to_path_buf(),
            run_id: "run-2002".to_string(),
            case_matrix: None,
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
    fn checked_in_case_retry_retains_terminal_lineage_and_current_source() -> Result<()> {
        let repository = clean_repository()?;
        let repository_commit =
            bijux_dna_containers::api::clean_repository_head_commit(repository.path())?;
        let matrix = case_matrix_for(repository.path(), &repository_commit)?;
        let prior = plan_benchmark_case_attempts(
            &matrix,
            "human-short-read-length-comparison",
            "attempt-001",
            "2026-08-10T00:00:00Z",
        )?;
        let temporary = bijux_dna_infra::temp_dir("benchmark-cycle-case-retry")?;
        let benchmark_root = temporary.path().join("evidence");
        crate::workflow::ensure_stage_campaign_attempts(&benchmark_root, &prior)?;
        for attempt in &prior.cycle.attempts {
            bijux_dna_runtime::record_attempt_superseded(
                &benchmark_root,
                &attempt.identity,
                "2026-08-10T00:01:00Z",
                "execution preparation authority was replaced",
            )?;
        }
        let matrix_path = temporary.path().join("case-matrix.json");
        let prior_plan_path = temporary.path().join("prior-plan.json");
        fs::write(&matrix_path, serde_json::to_vec_pretty(&matrix)?)?;
        fs::write(&prior_plan_path, serde_json::to_vec_pretty(&prior)?)?;
        let request = BenchmarkCycleRetryRequest {
            repository_root: repository.path().to_path_buf(),
            benchmark_root,
            case_matrix: matrix_path.clone(),
            prior_plan: prior_plan_path,
            prior_case_matrix: matrix_path,
            attempt_id: "attempt-002".to_string(),
            observed_at_utc: "2026-08-10T00:02:00Z".to_string(),
            output: temporary.path().join("retry-plan.json"),
        };

        let retry = retry_checked_in_benchmark_campaign(&request)?;
        assert_eq!(retry, retry_checked_in_benchmark_campaign(&request)?);
        let active = retry
            .cycle
            .attempts
            .iter()
            .filter(|attempt| attempt.identity.attempt_id == "attempt-002")
            .collect::<Vec<_>>();
        assert_eq!(active.len(), matrix.cases.len());
        assert!(active
            .iter()
            .all(|attempt| attempt.supersedes_attempt_id.as_deref() == Some("attempt-001")));
        assert_eq!(retry.repository_commit, repository_commit);
        Ok(())
    }

    #[test]
    fn status_reads_each_persisted_attempt_without_scheduler_observation() -> Result<()> {
        let repository = clean_repository()?;
        let temporary = bijux_dna_infra::temp_dir("benchmark-cycle-status")?;
        let output = temporary.path().join("run-2002-plan.json");
        let plan = plan_checked_in_benchmark_campaign(&BenchmarkCyclePlanRequest {
            repository_root: repository.path().to_path_buf(),
            run_id: "run-2002".to_string(),
            case_matrix: None,
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
            repository_root: repository.path().to_path_buf(),
            benchmark_root,
            plan: output,
            case_matrix: None,
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

    #[test]
    fn planning_rejects_dirty_source_even_when_the_registry_is_unchanged() -> Result<()> {
        let repository = clean_repository()?;
        fs::write(repository.path().join("uncommitted-source.rs"), "fn dirty() {}\n")?;
        let temporary = bijux_dna_infra::temp_dir("benchmark-cycle-dirty-source")?;
        let result = plan_checked_in_benchmark_campaign(&BenchmarkCyclePlanRequest {
            repository_root: repository.path().to_path_buf(),
            run_id: "run-2002".to_string(),
            case_matrix: None,
            campaign_id: "campaign-run-2002".to_string(),
            attempt_id: "attempt-0001".to_string(),
            observed_at_utc: "2026-08-10T00:00:00Z".to_string(),
            output: temporary.path().join("plan.json"),
        });
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn admission_reports_every_non_ready_attempt_without_mutation() -> Result<()> {
        let repository = clean_repository()?;
        let temporary = bijux_dna_infra::temp_dir("benchmark-cycle-admission")?;
        let output = temporary.path().join("run-2002-plan.json");
        let plan = plan_checked_in_benchmark_campaign(&BenchmarkCyclePlanRequest {
            repository_root: repository.path().to_path_buf(),
            run_id: "run-2002".to_string(),
            case_matrix: None,
            campaign_id: "campaign-run-2002".to_string(),
            attempt_id: "attempt-0001".to_string(),
            observed_at_utc: "2026-08-10T00:00:00Z".to_string(),
            output: output.clone(),
        })?;
        let benchmark_root = temporary.path().join("evidence");
        let study_root = temporary.path().join("study");
        fs::create_dir(&study_root)?;
        crate::workflow::ensure_stage_campaign_attempts(&benchmark_root, &plan)?;

        let admission = admit_checked_in_benchmark_campaign(&BenchmarkCycleAdmissionRequest {
            repository_root: repository.path().to_path_buf(),
            study_root,
            benchmark_root: benchmark_root.clone(),
            plan: output,
            case_matrix: None,
            admitted_at_utc: "2026-08-10T00:01:00Z".to_string(),
        })?;
        assert_eq!(admission.units.len(), 3);
        assert!(admission
            .units
            .iter()
            .all(|unit| unit.state == BenchmarkCycleAdmissionState::NotReady));
        for attempt in &plan.cycle.attempts {
            assert_eq!(
                bijux_dna_runtime::load_attempt_lifecycle(&benchmark_root, &attempt.identity)?
                    .current_status(),
                Some(BenchmarkLifecycleStatus::Planned)
            );
        }
        Ok(())
    }
}
