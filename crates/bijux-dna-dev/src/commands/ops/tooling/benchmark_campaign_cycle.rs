//! Thin operator composition for crate-owned benchmark campaign planning and persistence.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use chrono::{DateTime, Duration, Utc};

use self::lifecycle::{
    execution_schema, initialize, plan, plan_retry, plan_retry_from_tip,
    scheduler_observation_schema, schema, supersede,
};
use self::scheduler::{benchmark_accounting, benchmark_observation, lifecycle_status, slurm_state};
use super::{OpsCommandOutcome, Result, Workspace};

mod lifecycle;
mod scheduler;

const USAGE: &str = concat!(
    "Usage:\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- schema --output <path>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- execution-schema --output <path>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- scheduler-observation-schema --output <path>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- plan --run-id <run-id> --campaign-id <campaign-id> --attempt-id <attempt-id> --observed-at-utc <timestamp> --output <path> --repository-root <path>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- plan-retry --attempt-id <attempt-id> --benchmark-root <path> --observed-at-utc <timestamp> --output <path> --prior-plan <path> --repository-root <path>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- plan-retry-from-tip --attempt-id <attempt-id> --benchmark-root <path> --campaign-id <campaign-id> --observed-at-utc <timestamp> --output <path> --repository-root <path> --run-id <run-id> --tip-attempt-id <attempt-id>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- initialize --plan <path> --benchmark-root <path> --repository-root <path>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- supersede --benchmark-root <path> --observed-at-utc <timestamp> --plan <path> --reason <reason> --repository-root <path>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- prepare --account <account> --benchmark-root <path> --data-index <path> --inventory <path> --operator <name> --partition <partition> --plan <path> --prepared-at-utc <timestamp> --repository-root <path> --study-root <path>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- submit --account <account> --benchmark-root <path> --data-index <path> --inventory <path> --operator <name> --partition <partition> --plan <path> --prepared-at-utc <timestamp> --repository-root <path> --study-root <path> --submitted-at-utc <timestamp>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- reconcile --benchmark-root <path> --observed-at-utc <timestamp> --plan <path> --repository-root <path>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- admit --admitted-at-utc <timestamp> --benchmark-root <path> --plan <path> --repository-root <path> --study-root <path>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- publish-bam-coverage --benchmark-root <path> --output-root <path> --plan <path> --repository-root <path> --study-root <path>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- publish-bam-duplication-metrics --benchmark-root <path> --output-root <path> --plan <path> --repository-root <path> --study-root <path>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- publish-bam-endogenous-content --benchmark-root <path> --output-root <path> --plan <path> --repository-root <path> --study-root <path>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- publish-bam-genotyping --benchmark-root <path> --output-root <path> --plan <path> --repository-root <path> --study-root <path>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- publish-bam-mapping-summary --benchmark-root <path> --output-root <path> --plan <path> --repository-root <path> --study-root <path>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- publish-fastq-read-lengths --benchmark-root <path> --output-root <path> --plan <path> --repository-root <path> --study-root <path>\n",
    "  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- publish-fastq-profile-reads --benchmark-root <path> --output-root <path> --plan <path> --repository-root <path> --study-root <path>\n",
);
const MAX_PLAN_BYTES: usize = 16 * 1024 * 1024;

const BENCHMARK_PREPARATION_OPTIONS: &[&str] = &[
    "--account",
    "--benchmark-root",
    "--data-index",
    "--inventory",
    "--operator",
    "--partition",
    "--plan",
    "--prepared-at-utc",
    "--repository-root",
    "--study-root",
];

const BENCHMARK_SUBMISSION_OPTIONS: &[&str] = &[
    "--account",
    "--benchmark-root",
    "--data-index",
    "--inventory",
    "--operator",
    "--partition",
    "--plan",
    "--prepared-at-utc",
    "--repository-root",
    "--study-root",
    "--submitted-at-utc",
];

struct CampaignPreparation<'a> {
    repository_root: &'a Path,
    study_root: &'a Path,
    data_index: &'a Path,
    benchmark_root: &'a Path,
    plan: &'a bijux_dna_bench::BenchmarkCampaignPlan,
    authorities: &'a bijux_dna_bench::BenchmarkExecutionAuthorities,
    evidence: &'a CampaignEvidence,
    options: &'a BTreeMap<String, String>,
    scheduler_timestamp: &'a str,
    recover_submitted: bool,
}

enum CampaignEvidence {
    Bam(bijux_dna_bench::AdmittedBamCorpus),
    BamGenotyping(Box<bijux_dna_bench::AdmittedBamGenotypingCorpus>),
    ReadLengths(bijux_dna_bench::FastqReadLengthExpectationPublication),
    ProfileReads(bijux_dna_bench::FastqProfileReadsExpectationPublication),
}

impl CampaignPreparation<'_> {
    fn prepare_attempt(
        &self,
        identity: &bijux_dna_bench::BenchmarkAttemptIdentity,
    ) -> Result<bijux_dna_runner::api::PreparedBenchmarkSlurmJob> {
        let lifecycle = bijux_dna_bench::load_attempt_lifecycle(self.benchmark_root, identity)?;
        let spec = match lifecycle.current_status() {
            Some(bijux_dna_bench::BenchmarkLifecycleStatus::Planned) => {
                self.build_attempt(identity)?
            }
            Some(bijux_dna_bench::BenchmarkLifecycleStatus::Ready) => {
                let spec = bijux_dna_bench::load_execution_spec(self.benchmark_root, identity)?;
                self.validate_recovered_spec(&spec)?;
                spec
            }
            Some(bijux_dna_bench::BenchmarkLifecycleStatus::Submitted)
                if self.recover_submitted =>
            {
                let spec = bijux_dna_bench::load_execution_spec(self.benchmark_root, identity)?;
                self.validate_recovered_spec(&spec)?;
                spec
            }
            _ => bail!("benchmark operation requires an eligible planned or prepared attempt"),
        };
        bijux_dna_runner::api::prepare_benchmark_slurm_job(&spec, self.scheduler_timestamp)
    }

    fn build_attempt(
        &self,
        identity: &bijux_dna_bench::BenchmarkAttemptIdentity,
    ) -> Result<bijux_dna_bench::BenchmarkExecutionSpec> {
        let image = self.authorities.resolve_current_image(self.study_root, &identity.tool_id)?;
        let execution = match self.evidence {
            CampaignEvidence::BamGenotyping(corpus) => {
                bijux_dna_bench::prepare_bam_genotyping_material_from_corpus(
                    self.repository_root,
                    self.benchmark_root,
                    identity,
                    self.authorities.source_contract_sha256(),
                    required(self.options, "--operator")?,
                    required(self.options, "--prepared-at-utc")?,
                    required(self.options, "--account")?,
                    required(self.options, "--partition")?,
                    corpus,
                )?
                .execution
            }
            CampaignEvidence::Bam(corpus) => {
                let arguments = (
                    self.repository_root,
                    self.benchmark_root,
                    identity,
                    self.authorities.source_contract_sha256(),
                    required(self.options, "--operator")?,
                    required(self.options, "--prepared-at-utc")?,
                    required(self.options, "--account")?,
                    required(self.options, "--partition")?,
                    corpus,
                );
                match identity.run.run_id.as_str() {
                    "run-2002" => {
                        bijux_dna_bench::prepare_bam_coverage_material_from_corpus(
                            arguments.0,
                            arguments.1,
                            arguments.2,
                            arguments.3,
                            arguments.4,
                            arguments.5,
                            arguments.6,
                            arguments.7,
                            arguments.8,
                        )?
                        .execution
                    }
                    "run-2003" => {
                        bijux_dna_bench::prepare_bam_endogenous_content_material_from_corpus(
                            arguments.0,
                            arguments.1,
                            arguments.2,
                            arguments.3,
                            arguments.4,
                            arguments.5,
                            arguments.6,
                            arguments.7,
                            arguments.8,
                        )?
                        .execution
                    }
                    "run-2004" => {
                        bijux_dna_bench::prepare_bam_mapping_summary_material_from_corpus(
                            arguments.0,
                            arguments.1,
                            arguments.2,
                            arguments.3,
                            arguments.4,
                            arguments.5,
                            arguments.6,
                            arguments.7,
                            arguments.8,
                        )?
                        .execution
                    }
                    "run-2011" => {
                        bijux_dna_bench::prepare_bam_duplication_metrics_material_from_corpus(
                            arguments.0,
                            arguments.1,
                            arguments.2,
                            arguments.3,
                            arguments.4,
                            arguments.5,
                            arguments.6,
                            arguments.7,
                            arguments.8,
                        )?
                        .execution
                    }
                    _ => bail!("BAM campaign has no crate-owned executable material"),
                }
            }
            CampaignEvidence::ReadLengths(expectations) => {
                bijux_dna_bench::prepare_fastq_profile_read_lengths_material_from_expectations(
                    self.repository_root,
                    self.benchmark_root,
                    identity,
                    self.authorities.source_contract_sha256(),
                    required(self.options, "--operator")?,
                    required(self.options, "--prepared-at-utc")?,
                    required(self.options, "--account")?,
                    required(self.options, "--partition")?,
                    expectations,
                )?
                .execution
            }
            CampaignEvidence::ProfileReads(expectations) => {
                bijux_dna_bench::prepare_fastq_profile_reads_material_from_expectations(
                    self.repository_root,
                    self.benchmark_root,
                    identity,
                    self.authorities.source_contract_sha256(),
                    required(self.options, "--operator")?,
                    required(self.options, "--prepared-at-utc")?,
                    required(self.options, "--account")?,
                    required(self.options, "--partition")?,
                    expectations,
                )?
                .execution
            }
        };
        let spec = self.authorities.build_execution_spec(self.plan, identity, &image, execution)?;
        bijux_dna_bench::ensure_execution_spec(self.benchmark_root, &spec)?;
        Ok(spec)
    }

    fn validate_recovered_spec(
        &self,
        spec: &bijux_dna_bench::BenchmarkExecutionSpec,
    ) -> Result<()> {
        let image =
            self.authorities.resolve_current_image(self.study_root, &spec.identity.tool_id)?;
        let indexed_input = spec
            .inputs
            .iter()
            .find(|input| input.id == "corpus_data_index")
            .context("recovered benchmark specification omits its data index")?;
        if spec.source.repository_path != self.repository_root.display().to_string()
            || &spec.source.contract_sha256 != self.authorities.source_contract_sha256()
            || spec.image.path != image.path.display().to_string()
            || spec.image.sha256 != image.sha256
            || indexed_input.path != self.data_index.display().to_string()
            || spec.operator != required(self.options, "--operator")?
            || spec.prepared_at_utc != required(self.options, "--prepared-at-utc")?
            || spec.resources.account != required(self.options, "--account")?
            || spec.resources.partition != required(self.options, "--partition")?
        {
            bail!("recovered benchmark specification differs from requested authorities");
        }
        Ok(())
    }
}

pub(in super::super) fn tooling_benchmark_campaign_cycle(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let Some((operation, remaining)) = args.split_first() else {
        return Ok(OpsCommandOutcome::failure(USAGE));
    };
    let options = parse_options(remaining).context("parse benchmark campaign cycle options")?;
    validate_operation_timestamps(operation, &options)?;
    match operation.as_str() {
        "schema" => schema(workspace, &options),
        "execution-schema" => execution_schema(workspace, &options),
        "scheduler-observation-schema" => scheduler_observation_schema(workspace, &options),
        "plan" => plan(workspace, &options),
        "plan-retry" => plan_retry(workspace, &options),
        "plan-retry-from-tip" => plan_retry_from_tip(workspace, &options),
        "initialize" => initialize(workspace, &options),
        "supersede" => supersede(workspace, &options),
        "prepare" => prepare_campaign(workspace, &options),
        "submit" => submit_campaign(workspace, &options),
        "reconcile" => reconcile_campaign(workspace, &options),
        "admit" => admit_campaign(workspace, &options),
        "publish-bam-coverage" => publish_bam_coverage(workspace, &options),
        "publish-bam-duplication-metrics" => publish_bam_duplication_metrics(workspace, &options),
        "publish-bam-endogenous-content" => publish_bam_endogenous_content(workspace, &options),
        "publish-bam-genotyping" => publish_bam_genotyping(workspace, &options),
        "publish-bam-mapping-summary" => publish_bam_mapping_summary(workspace, &options),
        "publish-fastq-read-lengths" => publish_fastq_read_lengths(workspace, &options),
        "publish-fastq-profile-reads" => publish_fastq_profile_reads(workspace, &options),
        _ => Ok(OpsCommandOutcome::failure(USAGE)),
    }
}

fn prepare_campaign(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(options, BENCHMARK_PREPARATION_OPTIONS)?;
    let prepared =
        prepare_campaign_jobs(workspace, options, required(options, "--prepared-at-utc")?, false)?;
    let scripts = prepared
        .jobs
        .iter()
        .map(|(_, material)| material.request.script_path.display().to_string())
        .collect::<Vec<_>>()
        .join("\n");
    Ok(OpsCommandOutcome::success(format!(
        "prepared {} benchmark Slurm jobs under {}\n{}\n",
        prepared.jobs.len(),
        prepared.benchmark_root.display(),
        scripts
    )))
}

struct PreparedCampaign {
    benchmark_root: PathBuf,
    jobs: Vec<(
        bijux_dna_bench::BenchmarkAttemptIdentity,
        bijux_dna_runner::api::PreparedBenchmarkSlurmJob,
    )>,
}

fn submit_campaign(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(options, BENCHMARK_SUBMISSION_OPTIONS)?;
    let submitted_at_utc = required(options, "--submitted-at-utc")?;
    let prepared = prepare_campaign_jobs(workspace, options, submitted_at_utc, true)?;
    let scheduler = bijux_dna_runner::api::SlurmClientPaths::default();
    let mut submitted = Vec::new();
    for (identity, material) in &prepared.jobs {
        let handle = bijux_dna_runner::api::submit_prepared_benchmark_job(&scheduler, material)?;
        bijux_dna_runtime::record_attempt_submission(
            &prepared.benchmark_root,
            identity,
            submitted_at_utc,
            vec![bijux_dna_bench::BenchmarkJobReference {
                scheduler: "slurm".to_string(),
                job_id: handle.job_id.clone(),
            }],
        )?;
        submitted.push(format!("{}\t{}", identity.tool_id, handle.job_id));
    }
    Ok(OpsCommandOutcome::success(format!(
        "submitted {} benchmark Slurm jobs without polling under {}\n{}\n",
        submitted.len(),
        prepared.benchmark_root.display(),
        submitted.join("\n")
    )))
}

fn prepare_campaign_jobs(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
    scheduler_timestamp: &str,
    recover_submitted: bool,
) -> Result<PreparedCampaign> {
    let repository_root = resolve_path(workspace, required(options, "--repository-root")?)
        .canonicalize()
        .context("canonicalize benchmark source repository")?;
    let plan_path = resolve_path(workspace, required(options, "--plan")?);
    let plan = read_plan(&plan_path)?;
    let (repository_commit, campaign_registry_sha256) = source_identity_at(&repository_root)?;
    bijux_dna_bench::verify_campaign_plan_source(
        &plan,
        &repository_commit,
        &campaign_registry_sha256,
    )?;
    let benchmark_root = resolve_path(workspace, required(options, "--benchmark-root")?);
    bijux_dna_bench::ensure_stage_campaign_attempts(&benchmark_root, &plan)?;
    let benchmark_root =
        benchmark_root.canonicalize().context("canonicalize benchmark result root")?;
    let study_root = resolve_path(workspace, required(options, "--study-root")?)
        .canonicalize()
        .context("canonicalize STUDY root")?;
    let data_index = resolve_path(workspace, required(options, "--data-index")?)
        .canonicalize()
        .context("canonicalize benchmark data index")?;
    let inventory = resolve_path(workspace, required(options, "--inventory")?)
        .canonicalize()
        .context("canonicalize container inventory")?;
    let authorities = bijux_dna_bench::load_execution_authorities(
        &repository_root,
        &inventory,
        &repository_commit,
        &plan,
    )?;
    let run_id = campaign_run_id(&plan)?;
    if !has_crate_owned_campaign_material(run_id) {
        bail!("benchmark campaign has no crate-owned executable material: {run_id}");
    }
    let evidence = match run_id {
        "run-2001" => CampaignEvidence::BamGenotyping(Box::new(
            bijux_dna_bench::admit_bam_genotyping_corpus(&study_root, &data_index)?,
        )),
        "run-2002" | "run-2003" | "run-2004" | "run-2011" => {
            CampaignEvidence::Bam(bijux_dna_bench::admit_bam_benchmark_corpus(
                &study_root,
                &data_index,
                "ena-prjeb22390-err2112767-bam",
            )?)
        }
        "run-2042" | "run-2043" => {
            let read_lengths = bijux_dna_bench::ensure_fastq_read_length_expectations(
                &repository_root,
                &study_root,
                &data_index,
                "human-short-read-diversity",
                20,
                30,
            )?;
            if run_id == "run-2042" {
                CampaignEvidence::ReadLengths(read_lengths)
            } else {
                CampaignEvidence::ProfileReads(
                    bijux_dna_bench::ensure_fastq_profile_reads_expectations(
                        &study_root,
                        &read_lengths,
                    )?,
                )
            }
        }
        _ => bail!("benchmark campaign has no crate-owned executable material: {run_id}"),
    };
    let mut attempts = active_plan_attempts(&plan);
    attempts.sort_by(|left, right| left.identity.tool_id.cmp(&right.identity.tool_id));
    let preparation = CampaignPreparation {
        repository_root: &repository_root,
        study_root: &study_root,
        data_index: &data_index,
        benchmark_root: &benchmark_root,
        plan: &plan,
        authorities: &authorities,
        evidence: &evidence,
        options,
        scheduler_timestamp,
        recover_submitted,
    };
    let mut prepared = Vec::new();
    for attempt in &attempts {
        let scheduler = preparation.prepare_attempt(&attempt.identity)?;
        prepared.push((attempt.identity.clone(), scheduler));
    }
    for (identity, _) in &prepared {
        bijux_dna_runtime::record_attempt_ready(
            &benchmark_root,
            identity,
            required(options, "--prepared-at-utc")?,
        )?;
    }
    Ok(PreparedCampaign { benchmark_root, jobs: prepared })
}

fn has_crate_owned_campaign_material(run_id: &str) -> bool {
    matches!(
        run_id,
        "run-2001" | "run-2002" | "run-2003" | "run-2004" | "run-2011" | "run-2042" | "run-2043"
    )
}

fn reconcile_campaign(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(
        options,
        &["--benchmark-root", "--observed-at-utc", "--plan", "--repository-root"],
    )?;
    let observed_at_utc = required(options, "--observed-at-utc")?;
    let plan_path = resolve_path(workspace, required(options, "--plan")?);
    let plan = read_plan(&plan_path)?;
    let repository_root = resolve_path(workspace, required(options, "--repository-root")?)
        .canonicalize()
        .context("canonicalize benchmark source repository")?;
    let (repository_commit, campaign_registry_sha256) = source_identity_at(&repository_root)?;
    bijux_dna_bench::verify_campaign_plan_source(
        &plan,
        &repository_commit,
        &campaign_registry_sha256,
    )?;
    let benchmark_root = resolve_path(workspace, required(options, "--benchmark-root")?)
        .canonicalize()
        .context("canonicalize benchmark result root")?;
    let scheduler = bijux_dna_runner::api::SlurmClientPaths::default();
    let mut attempts = active_plan_attempts(&plan);
    attempts.sort_by(|left, right| left.identity.tool_id.cmp(&right.identity.tool_id));
    let mut observations = Vec::new();
    for attempt in attempts {
        let identity = &attempt.identity;
        let lifecycle = bijux_dna_bench::load_attempt_lifecycle(&benchmark_root, identity)?;
        let Some(status) = lifecycle.current_status() else {
            bail!("benchmark attempt has no current lifecycle status");
        };
        if !matches!(
            status,
            bijux_dna_bench::BenchmarkLifecycleStatus::Submitted
                | bijux_dna_bench::BenchmarkLifecycleStatus::Running
        ) {
            observations.push(format!(
                "{}\t{}\tnot_polled",
                identity.tool_id,
                lifecycle_status(status)
            ));
            continue;
        }
        let spec = bijux_dna_bench::load_execution_spec(&benchmark_root, identity)?;
        let handle = bijux_dna_runner::api::load_benchmark_slurm_job_handle(&spec)?;
        let observation =
            bijux_dna_runner::api::observe_slurm_job(&scheduler, &handle, observed_at_utc)?;
        bijux_dna_runtime::record_attempt_scheduler_observation(
            &benchmark_root,
            identity,
            benchmark_observation(&observation)?,
        )?;
        let job = bijux_dna_bench::BenchmarkJobReference {
            scheduler: "slurm".to_string(),
            job_id: handle.job_id.clone(),
        };
        if observation.state == bijux_dna_runner::api::SlurmJobState::Running {
            bijux_dna_runtime::record_attempt_running(
                &benchmark_root,
                identity,
                observed_at_utc,
                &job,
            )?;
        } else if observation.state.is_terminal() {
            let accounting = observation
                .accounting
                .as_ref()
                .context("terminal benchmark observation omits scheduler accounting")?;
            bijux_dna_runtime::record_attempt_accounting(
                &benchmark_root,
                identity,
                observed_at_utc,
                benchmark_accounting(accounting)?,
            )?;
        }
        observations.push(format!(
            "{}\t{}\t{}",
            identity.tool_id,
            handle.job_id,
            slurm_state(observation.state)
        ));
    }
    Ok(OpsCommandOutcome::success(format!(
        "reconciled {} benchmark attempts with at most one scheduler observation each\n{}\n",
        observations.len(),
        observations.join("\n")
    )))
}

fn admit_campaign(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(
        options,
        &["--admitted-at-utc", "--benchmark-root", "--plan", "--repository-root", "--study-root"],
    )?;
    let admitted_at_utc = required(options, "--admitted-at-utc")?;
    let plan_path = resolve_path(workspace, required(options, "--plan")?);
    let plan = read_plan(&plan_path)?;
    let repository_root = resolve_path(workspace, required(options, "--repository-root")?)
        .canonicalize()
        .context("canonicalize benchmark source repository")?;
    let (repository_commit, campaign_registry_sha256) = source_identity_at(&repository_root)?;
    bijux_dna_bench::verify_campaign_plan_source(
        &plan,
        &repository_commit,
        &campaign_registry_sha256,
    )?;
    let benchmark_root = resolve_path(workspace, required(options, "--benchmark-root")?)
        .canonicalize()
        .context("canonicalize benchmark result root")?;
    let study_root = resolve_path(workspace, required(options, "--study-root")?)
        .canonicalize()
        .context("canonicalize STUDY root")?;
    let mut attempts = active_plan_attempts(&plan);
    attempts.sort_by(|left, right| left.identity.tool_id.cmp(&right.identity.tool_id));
    let mut admitted = Vec::new();
    for attempt in attempts {
        let identity = &attempt.identity;
        let lifecycle = bijux_dna_bench::load_attempt_lifecycle(&benchmark_root, identity)?;
        if !matches!(
            lifecycle.current_status(),
            Some(
                bijux_dna_bench::BenchmarkLifecycleStatus::Collecting
                    | bijux_dna_bench::BenchmarkLifecycleStatus::Passed
            )
        ) {
            bail!("benchmark admission requires collecting or passed attempts");
        }
        let spec = bijux_dna_bench::load_execution_spec(&benchmark_root, identity)?;
        let publication = bijux_dna_bench::admit_benchmark_execution_outputs(
            &study_root,
            &spec,
            admitted_at_utc,
        )?;
        bijux_dna_runtime::record_attempt_acceptance(
            &benchmark_root,
            identity,
            admitted_at_utc,
            vec![publication.evidence.clone()],
        )?;
        admitted.push(format!("{}\t{}", identity.tool_id, publication.evidence.uri));
    }
    Ok(OpsCommandOutcome::success(format!(
        "admitted {} benchmark attempts from verified output closure\n{}\n",
        admitted.len(),
        admitted.join("\n")
    )))
}

fn publish_fastq_read_lengths(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(
        options,
        &["--benchmark-root", "--output-root", "--plan", "--repository-root", "--study-root"],
    )?;
    let plan_path = resolve_path(workspace, required(options, "--plan")?);
    let plan = read_plan(&plan_path)?;
    let benchmark_root = resolve_path(workspace, required(options, "--benchmark-root")?);
    let repository_root = resolve_path(workspace, required(options, "--repository-root")?);
    let study_root = resolve_path(workspace, required(options, "--study-root")?);
    let output_root = resolve_path(workspace, required(options, "--output-root")?);
    let publication = bijux_dna_bench::publish_fastq_profile_read_lengths_stage_report(
        &repository_root,
        &study_root,
        &benchmark_root,
        &plan,
        &output_root,
    )?;
    Ok(OpsCommandOutcome::success(format!(
        "published admitted FASTQ read-length stage report\njson\t{}\t{}\nmarkdown\t{}\t{}\n",
        publication.report_path.display(),
        publication.report_sha256,
        publication.markdown_path.display(),
        publication.markdown_sha256,
    )))
}

fn publish_fastq_profile_reads(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(
        options,
        &["--benchmark-root", "--output-root", "--plan", "--repository-root", "--study-root"],
    )?;
    let plan = read_plan(&resolve_path(workspace, required(options, "--plan")?))?;
    let publication = bijux_dna_bench::publish_fastq_profile_reads_stage_report(
        &resolve_path(workspace, required(options, "--repository-root")?),
        &resolve_path(workspace, required(options, "--study-root")?),
        &resolve_path(workspace, required(options, "--benchmark-root")?),
        &plan,
        &resolve_path(workspace, required(options, "--output-root")?),
    )?;
    Ok(OpsCommandOutcome::success(format!(
        "published admitted FASTQ profile stage report\njson\t{}\t{}\nmarkdown\t{}\t{}\n",
        publication.report_path.display(),
        publication.report_sha256,
        publication.markdown_path.display(),
        publication.markdown_sha256,
    )))
}

fn publish_bam_mapping_summary(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(
        options,
        &["--benchmark-root", "--output-root", "--plan", "--repository-root", "--study-root"],
    )?;
    let plan_path = resolve_path(workspace, required(options, "--plan")?);
    let plan = read_plan(&plan_path)?;
    let benchmark_root = resolve_path(workspace, required(options, "--benchmark-root")?);
    let repository_root = resolve_path(workspace, required(options, "--repository-root")?);
    let study_root = resolve_path(workspace, required(options, "--study-root")?);
    let output_root = resolve_path(workspace, required(options, "--output-root")?);
    let publication = bijux_dna_bench::publish_bam_mapping_summary_stage_report(
        &repository_root,
        &study_root,
        &benchmark_root,
        &plan,
        &output_root,
    )?;
    Ok(OpsCommandOutcome::success(format!(
        "published admitted BAM mapping-summary stage report\njson\t{}\t{}\nmarkdown\t{}\t{}\n",
        publication.report_path.display(),
        publication.report_sha256,
        publication.markdown_path.display(),
        publication.markdown_sha256,
    )))
}

fn publish_bam_genotyping(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    let arguments = report_arguments(workspace, options)?;
    let publication = bijux_dna_bench::publish_bam_genotyping_stage_report(
        &arguments.repository_root,
        &arguments.study_root,
        &arguments.benchmark_root,
        &arguments.plan,
        &arguments.output_root,
    )?;
    Ok(OpsCommandOutcome::success(format!(
        "published admitted BAM genotyping stage report\njson\t{}\t{}\nmarkdown\t{}\t{}\n",
        publication.report_path.display(),
        publication.report_sha256,
        publication.markdown_path.display(),
        publication.markdown_sha256,
    )))
}

fn publish_bam_coverage(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    let arguments = report_arguments(workspace, options)?;
    let publication = bijux_dna_bench::publish_bam_coverage_stage_report(
        &arguments.repository_root,
        &arguments.study_root,
        &arguments.benchmark_root,
        &arguments.plan,
        &arguments.output_root,
    )?;
    Ok(bam_observation_report_outcome("BAM coverage", &publication))
}

fn publish_bam_endogenous_content(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    let arguments = report_arguments(workspace, options)?;
    let publication = bijux_dna_bench::publish_bam_endogenous_content_stage_report(
        &arguments.repository_root,
        &arguments.study_root,
        &arguments.benchmark_root,
        &arguments.plan,
        &arguments.output_root,
    )?;
    Ok(bam_observation_report_outcome("BAM endogenous-content", &publication))
}

fn publish_bam_duplication_metrics(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    let arguments = report_arguments(workspace, options)?;
    let publication = bijux_dna_bench::publish_bam_duplication_metrics_stage_report(
        &arguments.repository_root,
        &arguments.study_root,
        &arguments.benchmark_root,
        &arguments.plan,
        &arguments.output_root,
    )?;
    Ok(bam_observation_report_outcome("BAM duplication-metrics", &publication))
}

struct StageReportArguments {
    repository_root: PathBuf,
    study_root: PathBuf,
    benchmark_root: PathBuf,
    output_root: PathBuf,
    plan: bijux_dna_bench::BenchmarkCampaignPlan,
}

fn report_arguments(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<StageReportArguments> {
    require_exact_options(
        options,
        &["--benchmark-root", "--output-root", "--plan", "--repository-root", "--study-root"],
    )?;
    Ok(StageReportArguments {
        repository_root: resolve_path(workspace, required(options, "--repository-root")?),
        study_root: resolve_path(workspace, required(options, "--study-root")?),
        benchmark_root: resolve_path(workspace, required(options, "--benchmark-root")?),
        output_root: resolve_path(workspace, required(options, "--output-root")?),
        plan: read_plan(&resolve_path(workspace, required(options, "--plan")?))?,
    })
}

fn bam_observation_report_outcome(
    name: &str,
    publication: &bijux_dna_bench::BamCoverageStageReportPublication,
) -> OpsCommandOutcome {
    OpsCommandOutcome::success(format!(
        "published admitted {name} stage report\njson\t{}\t{}\nmarkdown\t{}\t{}\n",
        publication.report_path.display(),
        publication.report_sha256,
        publication.markdown_path.display(),
        publication.markdown_sha256,
    ))
}

fn read_plan(path: &Path) -> Result<bijux_dna_bench::BenchmarkCampaignPlan> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_PLAN_BYTES)
        .with_context(|| format!("read benchmark campaign plan {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("parse benchmark campaign plan {}", path.display()))
}

fn source_identity_at(repository_root: &Path) -> Result<(String, String)> {
    let repository_commit =
        bijux_dna_environment::api::clean_repository_head_commit(repository_root)
            .context("require a clean source tree for benchmark campaign operation")?;
    let registry_path = repository_root.join("benchmarks/contracts/stage-campaigns.toml");
    let campaign_registry_sha256 = bijux_dna_infra::hash_file_sha256(&registry_path)
        .with_context(|| format!("hash benchmark campaign registry {}", registry_path.display()))?;
    Ok((repository_commit, campaign_registry_sha256))
}

fn active_plan_attempts(
    plan: &bijux_dna_bench::BenchmarkCampaignPlan,
) -> Vec<&bijux_dna_bench::BenchmarkAttemptRecordV2> {
    plan.cycle
        .attempts
        .iter()
        .filter(|attempt| {
            attempt.transitions.len() == 1
                && attempt.current_status()
                    == Some(bijux_dna_bench::BenchmarkLifecycleStatus::Planned)
        })
        .collect()
}

fn campaign_run_id(plan: &bijux_dna_bench::BenchmarkCampaignPlan) -> Result<&str> {
    let run_id = plan
        .cycle
        .attempts
        .first()
        .context("benchmark campaign plan has no attempts")?
        .identity
        .run
        .run_id
        .as_str();
    if plan.cycle.attempts.iter().any(|attempt| attempt.identity.run.run_id != run_id) {
        bail!("benchmark campaign plan mixes run identities");
    }
    Ok(run_id)
}

fn ensure_json_identity<T>(path: &Path, value: &T) -> Result<()>
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq,
{
    if path.exists() {
        let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_PLAN_BYTES)
            .with_context(|| format!("read existing benchmark campaign plan {}", path.display()))?;
        let existing: T = serde_json::from_slice(&bytes).with_context(|| {
            format!("parse existing benchmark campaign plan {}", path.display())
        })?;
        if existing != *value {
            bail!("benchmark campaign plan path is bound to different content: {}", path.display());
        }
        return Ok(());
    }
    bijux_dna_infra::atomic_create_json(path, value)
        .with_context(|| format!("write benchmark campaign plan {}", path.display()))
}

fn parse_options(args: &[String]) -> Result<BTreeMap<String, String>> {
    let mut options = BTreeMap::new();
    let mut chunks = args.chunks_exact(2);
    for pair in &mut chunks {
        if !pair[0].starts_with("--") || pair[1].starts_with("--") {
            bail!("benchmark campaign cycle options require --name value pairs\n{USAGE}");
        }
        if options.insert(pair[0].clone(), pair[1].clone()).is_some() {
            bail!("benchmark campaign cycle option is duplicated: {}", pair[0]);
        }
    }
    if !chunks.remainder().is_empty() {
        bail!("benchmark campaign cycle option lacks a value\n{USAGE}");
    }
    Ok(options)
}

fn require_exact_options(options: &BTreeMap<String, String>, expected: &[&str]) -> Result<()> {
    let actual = options.keys().map(String::as_str).collect::<Vec<_>>();
    if actual != expected {
        bail!("benchmark campaign cycle options differ from the operation contract\n{USAGE}");
    }
    Ok(())
}

fn required<'a>(options: &'a BTreeMap<String, String>, name: &str) -> Result<&'a str> {
    let value = options.get(name).with_context(|| format!("missing required option {name}"))?;
    if value.trim().is_empty() || value != value.trim() {
        bail!("benchmark campaign cycle option is empty or malformed: {name}");
    }
    Ok(value)
}

fn validate_operation_timestamps(
    operation: &str,
    options: &BTreeMap<String, String>,
) -> Result<()> {
    let names: &[&str] = match operation {
        "plan" | "plan-retry" | "plan-retry-from-tip" | "supersede" | "reconcile" => {
            &["--observed-at-utc"]
        }
        "prepare" => &["--prepared-at-utc"],
        "submit" => &["--prepared-at-utc", "--submitted-at-utc"],
        "admit" => &["--admitted-at-utc"],
        _ => return Ok(()),
    };
    let now = Utc::now();
    for name in names {
        validate_operator_timestamp_at(required(options, name)?, name, now)?;
    }
    Ok(())
}

fn validate_operator_timestamp_at(value: &str, name: &str, now: DateTime<Utc>) -> Result<()> {
    let parsed = DateTime::parse_from_rfc3339(value)
        .with_context(|| format!("benchmark campaign cycle timestamp is not RFC 3339: {name}"))?;
    if parsed.offset().local_minus_utc() != 0 {
        bail!("benchmark campaign cycle timestamp is not UTC: {name}");
    }
    if parsed.with_timezone(&Utc) > now + Duration::minutes(5) {
        bail!("benchmark campaign cycle timestamp is ahead of host UTC: {name}");
    }
    Ok(())
}

fn resolve_path(workspace: &Workspace, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace.root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::{
        has_crate_owned_campaign_material, parse_options, require_exact_options,
        scheduler::{benchmark_accounting, benchmark_observation},
        validate_operator_timestamp_at, BENCHMARK_PREPARATION_OPTIONS,
        BENCHMARK_SUBMISSION_OPTIONS,
    };

    #[test]
    fn campaign_material_dispatch_includes_duplication_metrics() {
        assert!(has_crate_owned_campaign_material("run-2001"));
        assert!(has_crate_owned_campaign_material("run-2011"));
        assert!(!has_crate_owned_campaign_material("run-9999"));
    }

    #[test]
    fn option_parser_rejects_duplicates_and_incomplete_pairs() {
        assert!(parse_options(&["--run-id".into(), "run-2042".into()]).is_ok());
        assert!(parse_options(&["--run-id".into()]).is_err());
        assert!(parse_options(&[
            "--run-id".into(),
            "run-2042".into(),
            "--run-id".into(),
            "run-2043".into(),
        ])
        .is_err());
    }

    #[test]
    fn operator_timestamp_guard_accepts_replay_and_small_clock_skew() -> anyhow::Result<()> {
        let now = Utc.with_ymd_and_hms(2026, 8, 5, 18, 0, 0).single().unwrap();
        validate_operator_timestamp_at("2025-01-01T00:00:00Z", "--observed-at-utc", now)?;
        validate_operator_timestamp_at("2026-08-05T18:05:00Z", "--observed-at-utc", now)?;
        Ok(())
    }

    #[test]
    fn operator_timestamp_guard_rejects_future_non_utc_and_malformed_values() {
        let now = Utc.with_ymd_and_hms(2026, 8, 5, 18, 0, 0).single().unwrap();
        assert!(validate_operator_timestamp_at("2026-08-05T18:05:01Z", "--submitted-at-utc", now)
            .is_err());
        assert!(validate_operator_timestamp_at(
            "2026-08-05T20:00:00+02:00",
            "--submitted-at-utc",
            now
        )
        .is_err());
        assert!(validate_operator_timestamp_at("not-a-time", "--submitted-at-utc", now).is_err());
    }

    #[test]
    fn operation_contract_rejects_unknown_options() -> anyhow::Result<()> {
        let options = parse_options(&[
            "--plan".into(),
            "plan.json".into(),
            "--unknown".into(),
            "value".into(),
        ])?;
        assert!(require_exact_options(&options, &["--benchmark-root", "--plan"]).is_err());
        Ok(())
    }

    #[test]
    fn campaign_preparation_contract_requires_complete_authorities() -> anyhow::Result<()> {
        let options = parse_options(&[
            "--account".into(),
            "project".into(),
            "--benchmark-root".into(),
            "/study/benchmarks".into(),
            "--data-index".into(),
            "/study/data-index.json".into(),
            "--inventory".into(),
            "/study/image-inventory.json".into(),
            "--operator".into(),
            "operator".into(),
            "--partition".into(),
            "compute".into(),
            "--plan".into(),
            "/study/plan.json".into(),
            "--prepared-at-utc".into(),
            "2026-08-05T02:00:00Z".into(),
            "--repository-root".into(),
            "/source/bijux-genomics".into(),
            "--study-root".into(),
            "/study".into(),
        ])?;
        assert!(require_exact_options(&options, BENCHMARK_PREPARATION_OPTIONS).is_ok());
        Ok(())
    }

    #[test]
    fn campaign_submission_contract_requires_separate_submission_time() -> anyhow::Result<()> {
        let options = parse_options(&[
            "--account".into(),
            "project".into(),
            "--benchmark-root".into(),
            "/study/benchmarks".into(),
            "--data-index".into(),
            "/study/data-index.json".into(),
            "--inventory".into(),
            "/study/image-inventory.json".into(),
            "--operator".into(),
            "operator".into(),
            "--partition".into(),
            "compute".into(),
            "--plan".into(),
            "/study/plan.json".into(),
            "--prepared-at-utc".into(),
            "2026-08-05T02:00:00Z".into(),
            "--repository-root".into(),
            "/source/bijux-genomics".into(),
            "--study-root".into(),
            "/study".into(),
            "--submitted-at-utc".into(),
            "2026-08-05T03:00:00Z".into(),
        ])?;
        assert!(require_exact_options(&options, BENCHMARK_SUBMISSION_OPTIONS).is_ok());
        Ok(())
    }

    #[test]
    fn scheduler_accounting_maps_terminal_evidence_without_loss() -> anyhow::Result<()> {
        let accounting = bijux_dna_runner::api::SlurmAccounting {
            job_id: "456789".to_string(),
            state: bijux_dna_runner::api::SlurmJobState::OutOfMemory,
            raw_state: "OUT_OF_MEMORY".to_string(),
            exit_code: 1,
            signal: 9,
            elapsed_seconds: 12,
            total_cpu_seconds: 17,
            max_rss_bytes: Some(4096),
            node_list: "node1".to_string(),
            reason: Some("memory limit".to_string()),
        };
        let mapped = benchmark_accounting(&accounting)?;
        assert_eq!(mapped.job_id, accounting.job_id);
        assert_eq!(mapped.raw_state, accounting.raw_state);
        assert_eq!(mapped.max_rss_bytes, accounting.max_rss_bytes);
        assert_eq!(mapped.state, bijux_dna_bench::BenchmarkSchedulerTerminalState::OutOfMemory);
        Ok(())
    }

    #[test]
    fn scheduler_observation_maps_repeated_queue_evidence_without_lifecycle_reduction(
    ) -> anyhow::Result<()> {
        let observation = bijux_dna_runner::api::SlurmJobObservation {
            job_id: "456789".to_string(),
            state: bijux_dna_runner::api::SlurmJobState::Running,
            source: bijux_dna_runner::api::SlurmEvidenceSource::Queue,
            observed_at_utc: "2026-08-05T04:45:06Z".to_string(),
            accounting: None,
        };
        let mapped = benchmark_observation(&observation)?;
        assert_eq!(mapped.job_id, observation.job_id);
        assert_eq!(mapped.observed_at_utc, observation.observed_at_utc);
        assert_eq!(mapped.state, bijux_dna_bench::BenchmarkSchedulerObservedState::Running);
        assert_eq!(mapped.source, bijux_dna_bench::BenchmarkSchedulerEvidenceSource::Queue);
        assert!(mapped.accounting.is_none());
        Ok(())
    }

    #[test]
    fn fastq_admission_contract_requires_study_authority() -> anyhow::Result<()> {
        let options = parse_options(&[
            "--admitted-at-utc".into(),
            "2026-08-05T04:00:00Z".into(),
            "--benchmark-root".into(),
            "/study/results/benchmarks".into(),
            "--plan".into(),
            "/study/plan.json".into(),
            "--repository-root".into(),
            "/source/bijux-genomics".into(),
            "--study-root".into(),
            "/study".into(),
        ])?;
        assert!(require_exact_options(
            &options,
            &[
                "--admitted-at-utc",
                "--benchmark-root",
                "--plan",
                "--repository-root",
                "--study-root"
            ]
        )
        .is_ok());
        Ok(())
    }
}
