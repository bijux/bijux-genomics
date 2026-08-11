//! Owner: bijux-dna-bench
//! Admitted attempt, scheduler, source, and image evidence for stage reports.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkCampaignPlan, BenchmarkExecutionInput, BenchmarkExecutionSpec,
    BenchmarkLifecycleStatus, BenchmarkReportDownstreamProbe, BenchmarkReportImage,
    BenchmarkReportInputLock, BenchmarkReportSlurmJob, BenchmarkReportStageInterface,
    BenchmarkSchedulerTerminalState, BenchmarkStageCampaignSpec,
};

use super::{campaign_threads, hash_json, read_json};
use crate::{load_attempt_lifecycle, load_execution_spec};

#[derive(Debug, Clone)]
pub(in crate::workflow) struct AdmittedAttemptEvidence {
    pub(in crate::workflow) spec: BenchmarkExecutionSpec,
    pub(in crate::workflow) source_evidence_sha256: String,
    pub(in crate::workflow) image: BenchmarkReportImage,
    pub(in crate::workflow) slurm: BenchmarkReportSlurmJob,
    pub(in crate::workflow) input_locks: Vec<BenchmarkReportInputLock>,
    pub(in crate::workflow) input_lock_sha256: String,
    pub(in crate::workflow) parameter_lock_sha256: String,
    pub(in crate::workflow) interface: BenchmarkReportStageInterface,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::workflow) fn load_admitted_attempt_evidence(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    campaign: &BenchmarkStageCampaignSpec,
    plan: &BenchmarkCampaignPlan,
    identity: &bijux_dna_bench_model::BenchmarkAttemptIdentity,
) -> Result<AdmittedAttemptEvidence> {
    let lifecycle = load_attempt_lifecycle(benchmark_root, identity)?;
    if lifecycle.current_status() != Some(BenchmarkLifecycleStatus::Passed) {
        bail!("stage report requires every planned attempt to pass");
    }
    let accounting = lifecycle
        .transitions
        .iter()
        .flat_map(|transition| &transition.accounting)
        .collect::<Vec<_>>();
    if accounting.len() != 1
        || accounting[0].state != BenchmarkSchedulerTerminalState::Completed
        || accounting[0].exit_code != 0
        || accounting[0].signal != 0
    {
        bail!("benchmark attempt lacks one successful terminal accounting row");
    }
    let jobs =
        lifecycle.transitions.iter().flat_map(|transition| &transition.jobs).collect::<Vec<_>>();
    if jobs.len() != 1 || jobs[0].scheduler != "slurm" || jobs[0].job_id != accounting[0].job_id {
        bail!("benchmark attempt scheduler identity is inconsistent");
    }

    let spec = load_execution_spec(benchmark_root, identity)?;
    validate_attempt_spec(repository_root, benchmark_root, campaign, plan, identity, &spec)?;
    let admission_path = Path::new(&spec.outputs.result_root).join("admission/acceptance.json");
    let admission: crate::BenchmarkExecutionAdmission = read_json(&admission_path)?;
    let replay =
        crate::admit_benchmark_execution_outputs(study_root, &spec, &admission.admitted_at_utc)?;
    if replay.report != admission {
        bail!("replayed benchmark admission differs from retained acceptance evidence");
    }
    let retained_evidence = lifecycle
        .transitions
        .iter()
        .flat_map(|transition| transition.evidence.iter())
        .collect::<Vec<_>>();
    if retained_evidence != [&replay.evidence] {
        bail!("benchmark lifecycle does not retain exactly its acceptance publication");
    }

    let input_locks = input_locks(&spec.inputs)?;
    let input_lock_sha256 = hash_json(&input_locks)?;
    let parameter_lock_sha256 = hash_json(&spec.parameters)?;
    let interface = report_interface(&spec);
    let image_path = Path::new(&spec.image.path);
    let image_metadata = fs::symlink_metadata(image_path)
        .with_context(|| format!("inspect benchmark image {}", image_path.display()))?;
    if image_metadata.file_type().is_symlink() || !image_metadata.is_file() {
        bail!("benchmark image is not a regular non-symlink file");
    }
    if bijux_dna_infra::hash_file_sha256(image_path)? != spec.image.sha256 {
        bail!("benchmark image changed after execution");
    }
    let image_name = image_path
        .file_name()
        .and_then(|name| name.to_str())
        .context("benchmark image has no UTF-8 file name")?
        .to_string();
    Ok(AdmittedAttemptEvidence {
        source_evidence_sha256: replay.evidence.sha256,
        image: BenchmarkReportImage {
            name: image_name,
            sha256: spec.image.sha256.clone(),
            size_bytes: image_metadata.len(),
        },
        slurm: BenchmarkReportSlurmJob {
            job_id: accounting[0].job_id.clone(),
            host: accounting[0].node_list.clone(),
            elapsed: format_elapsed(accounting[0].elapsed_seconds),
            allocated_cpus: spec.resources.cpus.to_string(),
        },
        input_locks,
        input_lock_sha256,
        parameter_lock_sha256,
        interface,
        spec,
    })
}

fn validate_attempt_spec(
    repository_root: &Path,
    benchmark_root: &Path,
    campaign: &BenchmarkStageCampaignSpec,
    plan: &BenchmarkCampaignPlan,
    identity: &bijux_dna_bench_model::BenchmarkAttemptIdentity,
    spec: &BenchmarkExecutionSpec,
) -> Result<()> {
    let attempt_root = crate::benchmark_attempt_directory(benchmark_root, identity)?
        .canonicalize()
        .context("canonicalize benchmark attempt root")?;
    let result_root = Path::new(&spec.outputs.result_root)
        .canonicalize()
        .context("canonicalize benchmark specification result root")?;
    validate_recorded_repository_location(repository_root, &spec.source.repository_path)?;
    let source_commit = bijux_dna_containers::api::clean_repository_head_commit(repository_root)
        .context("verify benchmark report source repository")?;
    if spec.identity != *identity
        || result_root != attempt_root
        || source_commit != spec.source.repository_commit
        || spec.source.repository_commit != plan.repository_commit
        || spec.measurement.warmup_repetitions != campaign.warmup_repetitions
        || spec.measurement.measured_repetitions != campaign.measured_repetitions
        || spec.resources.cpus != *campaign_threads(campaign)?.get(&identity.tool_id).unwrap_or(&0)
        || spec
            .source
            .contract_sha256
            .get("campaign_registry")
            .is_none_or(|digest| digest != &plan.campaign_registry_sha256)
    {
        bail!("benchmark execution specification drifts from campaign authority");
    }
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(
        &repository_root.join("containers/contracts/catalog.toml"),
    )?;
    if spec
        .source
        .contract_sha256
        .get("container_catalog")
        .is_none_or(|digest| digest != &catalog_sha256)
    {
        bail!("benchmark execution catalog lock differs from source");
    }
    Ok(())
}

fn validate_recorded_repository_location(
    repository_root: &Path,
    recorded_repository_path: &str,
) -> Result<()> {
    let recorded_path = Path::new(recorded_repository_path);
    match fs::symlink_metadata(recorded_path) {
        Ok(_) => {
            let recorded_root = recorded_path
                .canonicalize()
                .context("canonicalize recorded benchmark specification repository")?;
            if recorded_root != repository_root {
                bail!("recorded benchmark repository differs from publication authority");
            }
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).context("inspect recorded benchmark specification repository"),
    }
}

fn input_locks(inputs: &[BenchmarkExecutionInput]) -> Result<Vec<BenchmarkReportInputLock>> {
    let mut locks = inputs
        .iter()
        .map(|input| {
            let kind = serde_json::to_value(input.kind)?
                .as_str()
                .context("benchmark input kind is not a string")?
                .to_string();
            Ok(BenchmarkReportInputLock {
                id: input.id.clone(),
                kind,
                mount_path: input.mount_path.clone(),
                sha256: input.sha256.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    locks.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(locks)
}

fn report_interface(spec: &BenchmarkExecutionSpec) -> BenchmarkReportStageInterface {
    BenchmarkReportStageInterface {
        accepted_input_contracts: spec.interface.accepted_input_contracts.clone(),
        output_contract: spec.interface.output_contract.clone(),
        normalized_result_contract: spec.interface.normalized_result_contract.clone(),
        downstream_probe: BenchmarkReportDownstreamProbe {
            stage_id: spec.interface.downstream_probe.stage_id.clone(),
            input_contract: spec.interface.downstream_probe.input_contract.clone(),
            source: spec.interface.downstream_probe.source.clone(),
            artifact: spec.interface.downstream_probe.artifact.clone(),
        },
    }
}

fn format_elapsed(seconds: u64) -> String {
    format!("{:02}:{:02}:{:02}", seconds / 3600, seconds % 3600 / 60, seconds % 60)
}

#[cfg(test)]
mod tests {
    use super::{format_elapsed, validate_recorded_repository_location};

    #[test]
    fn elapsed_accounting_renders_without_day_rollover() {
        assert_eq!(format_elapsed(90_061), "25:01:01");
    }

    #[test]
    fn missing_recorded_repository_path_is_non_authoritative() -> anyhow::Result<()> {
        let authority = bijux_dna_infra::temp_dir("benchmark-report-source-authority")?;
        let missing = authority.path().join("retired-execution-checkout");
        validate_recorded_repository_location(
            authority.path(),
            missing.to_str().ok_or_else(|| anyhow::anyhow!("temporary path is not UTF-8"))?,
        )
    }

    #[test]
    fn existing_recorded_repository_path_must_match_authority() -> anyhow::Result<()> {
        let authority = bijux_dna_infra::temp_dir("benchmark-report-source-mismatch")?;
        let recorded = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(validate_recorded_repository_location(
            authority.path(),
            recorded.to_str().ok_or_else(|| anyhow::anyhow!("manifest path is not UTF-8"))?,
        )
        .is_err());
        Ok(())
    }
}
