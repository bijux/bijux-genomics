//! Thin operator composition for crate-owned benchmark campaign planning and persistence.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};

use super::{OpsCommandOutcome, Result, Workspace};

const USAGE: &str = "Usage:\n  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- schema --output <path>\n  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- execution-schema --output <path>\n  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- plan --run-id <run-id> --campaign-id <campaign-id> --attempt-id <attempt-id> --observed-at-utc <timestamp> --output <path>\n  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- plan-retry --attempt-id <attempt-id> --benchmark-root <path> --observed-at-utc <timestamp> --output <path> --prior-plan <path> --repository-root <path>\n  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- initialize --plan <path> --benchmark-root <path>\n  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- supersede --benchmark-root <path> --observed-at-utc <timestamp> --plan <path> --reason <reason> --repository-root <path>\n  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- prepare-fastq-read-lengths --account <account> --benchmark-root <path> --data-index <path> --inventory <path> --operator <name> --partition <partition> --plan <path> --prepared-at-utc <timestamp> --study-root <path>\n  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- submit-fastq-read-lengths --account <account> --benchmark-root <path> --data-index <path> --inventory <path> --operator <name> --partition <partition> --plan <path> --prepared-at-utc <timestamp> --study-root <path> --submitted-at-utc <timestamp>\n  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- reconcile-fastq-read-lengths --benchmark-root <path> --observed-at-utc <timestamp> --plan <path>\n  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- admit-fastq-read-lengths --admitted-at-utc <timestamp> --benchmark-root <path> --plan <path> --study-root <path>\n  cargo run -p bijux-dna-dev -- tooling run benchmark-campaign-cycle -- publish-fastq-read-lengths --benchmark-root <path> --output-root <path> --plan <path> --repository-root <path> --study-root <path>\n";
const MAX_PLAN_BYTES: usize = 16 * 1024 * 1024;

const FASTQ_PREPARATION_OPTIONS: &[&str] = &[
    "--account",
    "--benchmark-root",
    "--data-index",
    "--inventory",
    "--operator",
    "--partition",
    "--plan",
    "--prepared-at-utc",
    "--study-root",
];

const FASTQ_SUBMISSION_OPTIONS: &[&str] = &[
    "--account",
    "--benchmark-root",
    "--data-index",
    "--inventory",
    "--operator",
    "--partition",
    "--plan",
    "--prepared-at-utc",
    "--study-root",
    "--submitted-at-utc",
];

struct FastqPreparation<'a> {
    repository_root: &'a Path,
    study_root: &'a Path,
    data_index: &'a Path,
    benchmark_root: &'a Path,
    plan: &'a bijux_dna_bench::BenchmarkCampaignPlan,
    authorities: &'a bijux_dna_bench::BenchmarkExecutionAuthorities,
    expectations: &'a bijux_dna_bench::FastqReadLengthExpectationPublication,
    options: &'a BTreeMap<String, String>,
    scheduler_timestamp: &'a str,
    recover_submitted: bool,
}

impl FastqPreparation<'_> {
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
        let material =
            bijux_dna_bench::prepare_fastq_profile_read_lengths_material_from_expectations(
                self.repository_root,
                self.benchmark_root,
                identity,
                self.authorities.source_contract_sha256(),
                required(self.options, "--operator")?,
                required(self.options, "--prepared-at-utc")?,
                required(self.options, "--account")?,
                required(self.options, "--partition")?,
                self.expectations,
            )?;
        let spec = self.authorities.build_execution_spec(
            self.plan,
            identity,
            &image,
            material.execution,
        )?;
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
    match operation.as_str() {
        "schema" => schema(workspace, &options),
        "execution-schema" => execution_schema(workspace, &options),
        "plan" => plan(workspace, &options),
        "plan-retry" => plan_retry(workspace, &options),
        "initialize" => initialize(workspace, &options),
        "supersede" => supersede(workspace, &options),
        "prepare-fastq-read-lengths" => prepare_fastq_read_lengths(workspace, &options),
        "submit-fastq-read-lengths" => submit_fastq_read_lengths(workspace, &options),
        "reconcile-fastq-read-lengths" => reconcile_fastq_read_lengths(workspace, &options),
        "admit-fastq-read-lengths" => admit_fastq_read_lengths(workspace, &options),
        "publish-fastq-read-lengths" => publish_fastq_read_lengths(workspace, &options),
        _ => Ok(OpsCommandOutcome::failure(USAGE)),
    }
}

fn prepare_fastq_read_lengths(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(options, FASTQ_PREPARATION_OPTIONS)?;
    let prepared =
        prepare_fastq_campaign(workspace, options, required(options, "--prepared-at-utc")?, false)?;
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

struct PreparedFastqCampaign {
    benchmark_root: PathBuf,
    jobs: Vec<(
        bijux_dna_bench::BenchmarkAttemptIdentity,
        bijux_dna_runner::api::PreparedBenchmarkSlurmJob,
    )>,
}

fn submit_fastq_read_lengths(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(options, FASTQ_SUBMISSION_OPTIONS)?;
    let submitted_at_utc = required(options, "--submitted-at-utc")?;
    let prepared = prepare_fastq_campaign(workspace, options, submitted_at_utc, true)?;
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

fn prepare_fastq_campaign(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
    scheduler_timestamp: &str,
    recover_submitted: bool,
) -> Result<PreparedFastqCampaign> {
    let repository_root =
        workspace.root.canonicalize().context("canonicalize benchmark source repository")?;
    let plan_path = resolve_path(workspace, required(options, "--plan")?);
    let plan = read_plan(&plan_path)?;
    let (repository_commit, campaign_registry_sha256) = source_identity(workspace)?;
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
    let expectations = bijux_dna_bench::ensure_fastq_read_length_expectations(
        &repository_root,
        &study_root,
        &data_index,
        "human-short-read-diversity",
        20,
        30,
    )?;
    let mut attempts = active_plan_attempts(&plan);
    attempts.sort_by(|left, right| left.identity.tool_id.cmp(&right.identity.tool_id));
    let preparation = FastqPreparation {
        repository_root: &repository_root,
        study_root: &study_root,
        data_index: &data_index,
        benchmark_root: &benchmark_root,
        plan: &plan,
        authorities: &authorities,
        expectations: &expectations,
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
    Ok(PreparedFastqCampaign { benchmark_root, jobs: prepared })
}

fn reconcile_fastq_read_lengths(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(options, &["--benchmark-root", "--observed-at-utc", "--plan"])?;
    let observed_at_utc = required(options, "--observed-at-utc")?;
    let plan_path = resolve_path(workspace, required(options, "--plan")?);
    let plan = read_plan(&plan_path)?;
    let (repository_commit, campaign_registry_sha256) = source_identity(workspace)?;
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

fn admit_fastq_read_lengths(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(
        options,
        &["--admitted-at-utc", "--benchmark-root", "--plan", "--study-root"],
    )?;
    let admitted_at_utc = required(options, "--admitted-at-utc")?;
    let plan_path = resolve_path(workspace, required(options, "--plan")?);
    let plan = read_plan(&plan_path)?;
    let (repository_commit, campaign_registry_sha256) = source_identity(workspace)?;
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

fn benchmark_accounting(
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

fn slurm_state(state: bijux_dna_runner::api::SlurmJobState) -> &'static str {
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

fn lifecycle_status(status: bijux_dna_bench::BenchmarkLifecycleStatus) -> &'static str {
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

fn execution_schema(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(options, &["--output"])?;
    let output = resolve_path(workspace, required(options, "--output")?);
    let schema = serde_json::to_value(bijux_dna_bench::benchmark_execution_spec_schema())?;
    ensure_json_identity(&output, &schema)?;
    Ok(OpsCommandOutcome::success(format!(
        "published benchmark execution specification schema at {}\n",
        output.display()
    )))
}

fn schema(workspace: &Workspace, options: &BTreeMap<String, String>) -> Result<OpsCommandOutcome> {
    require_exact_options(options, &["--output"])?;
    let output = resolve_path(workspace, required(options, "--output")?);
    let schema = serde_json::to_value(bijux_dna_bench::benchmark_campaign_plan_schema())?;
    ensure_json_identity(&output, &schema)?;
    Ok(OpsCommandOutcome::success(format!(
        "published benchmark campaign plan schema at {}\n",
        output.display()
    )))
}

fn plan(workspace: &Workspace, options: &BTreeMap<String, String>) -> Result<OpsCommandOutcome> {
    require_exact_options(
        options,
        &["--attempt-id", "--campaign-id", "--observed-at-utc", "--output", "--run-id"],
    )?;
    let (repository_commit, campaign_registry_sha256) = source_identity(workspace)?;
    let (registry, _, _) = bijux_dna_bench::load_checked_in_benchmark_planning_contracts()?;
    let cycle = bijux_dna_bench::plan_stage_campaign_attempts(
        &registry,
        &repository_commit,
        &campaign_registry_sha256,
        required(options, "--run-id")?,
        required(options, "--campaign-id")?,
        required(options, "--attempt-id")?,
        required(options, "--observed-at-utc")?,
    )?;
    let output = resolve_path(workspace, required(options, "--output")?);
    ensure_json_identity(&output, &cycle)?;
    Ok(OpsCommandOutcome::success(format!(
        "planned {} benchmark attempts at {}\n",
        cycle.cycle.attempts.len(),
        output.display()
    )))
}

fn plan_retry(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(
        options,
        &[
            "--attempt-id",
            "--benchmark-root",
            "--observed-at-utc",
            "--output",
            "--prior-plan",
            "--repository-root",
        ],
    )?;
    let repository_root = resolve_path(workspace, required(options, "--repository-root")?)
        .canonicalize()
        .context("canonicalize benchmark retry repository")?;
    let (repository_commit, campaign_registry_sha256) = source_identity_at(&repository_root)?;
    let prior_plan_path = resolve_path(workspace, required(options, "--prior-plan")?);
    let prior_plan = read_plan(&prior_plan_path)?;
    bijux_dna_bench::verify_campaign_plan_source(
        &prior_plan,
        &repository_commit,
        &campaign_registry_sha256,
    )?;
    let benchmark_root = resolve_path(workspace, required(options, "--benchmark-root")?)
        .canonicalize()
        .context("canonicalize benchmark retry result root")?;
    let prior_attempts = prior_plan
        .cycle
        .attempts
        .iter()
        .map(|attempt| bijux_dna_bench::load_attempt_lifecycle(&benchmark_root, &attempt.identity))
        .collect::<Result<Vec<_>>>()?;
    let registry = bijux_dna_bench::load_stage_campaign_registry(
        &repository_root.join("benchmarks/contracts/stage-campaigns.toml"),
    )?;
    let retry = bijux_dna_bench::plan_stage_campaign_retry_attempts(
        &registry,
        &repository_commit,
        &campaign_registry_sha256,
        prior_attempts,
        required(options, "--attempt-id")?,
        required(options, "--observed-at-utc")?,
    )?;
    let output = resolve_path(workspace, required(options, "--output")?);
    ensure_json_identity(&output, &retry)?;
    Ok(OpsCommandOutcome::success(format!(
        "planned {} active benchmark retries with {} retained lineage records at {}\n",
        active_plan_attempts(&retry).len(),
        retry.cycle.attempts.len() - active_plan_attempts(&retry).len(),
        output.display()
    )))
}

fn supersede(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(
        options,
        &["--benchmark-root", "--observed-at-utc", "--plan", "--reason", "--repository-root"],
    )?;
    let repository_root = resolve_path(workspace, required(options, "--repository-root")?)
        .canonicalize()
        .context("canonicalize benchmark supersession repository")?;
    let (repository_commit, campaign_registry_sha256) = source_identity_at(&repository_root)?;
    let plan_path = resolve_path(workspace, required(options, "--plan")?);
    let plan = read_plan(&plan_path)?;
    bijux_dna_bench::verify_campaign_plan_source(
        &plan,
        &repository_commit,
        &campaign_registry_sha256,
    )?;
    let benchmark_root = resolve_path(workspace, required(options, "--benchmark-root")?)
        .canonicalize()
        .context("canonicalize benchmark supersession root")?;
    let attempts = active_plan_attempts(&plan);
    if attempts.is_empty() {
        bail!("benchmark supersession plan has no active attempt origins");
    }
    let reason = required(options, "--reason")?;
    for attempt in &attempts {
        let persisted =
            bijux_dna_bench::load_attempt_lifecycle(&benchmark_root, &attempt.identity)?;
        let is_active = matches!(
            persisted.current_status(),
            Some(
                bijux_dna_bench::BenchmarkLifecycleStatus::Planned
                    | bijux_dna_bench::BenchmarkLifecycleStatus::Ready
                    | bijux_dna_bench::BenchmarkLifecycleStatus::Submitted
                    | bijux_dna_bench::BenchmarkLifecycleStatus::Running
                    | bijux_dna_bench::BenchmarkLifecycleStatus::Collecting
            )
        );
        let is_identical_replay = persisted.current_status()
            == Some(bijux_dna_bench::BenchmarkLifecycleStatus::Superseded)
            && persisted.transitions.last().and_then(|transition| transition.reason.as_deref())
                == Some(reason);
        if !is_active && !is_identical_replay {
            bail!("benchmark supersession requires active persisted attempts");
        }
    }
    for attempt in &attempts {
        bijux_dna_runtime::record_attempt_superseded(
            &benchmark_root,
            &attempt.identity,
            required(options, "--observed-at-utc")?,
            reason,
        )?;
    }
    Ok(OpsCommandOutcome::success(format!(
        "ensured {} benchmark attempts are superseded with the retained reason\n",
        attempts.len()
    )))
}

fn initialize(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(options, &["--benchmark-root", "--plan"])?;
    let plan_path = resolve_path(workspace, required(options, "--plan")?);
    let cycle = read_plan(&plan_path)?;
    let (repository_commit, campaign_registry_sha256) = source_identity(workspace)?;
    bijux_dna_bench::verify_campaign_plan_source(
        &cycle,
        &repository_commit,
        &campaign_registry_sha256,
    )?;
    let benchmark_root = resolve_path(workspace, required(options, "--benchmark-root")?);
    let directories = bijux_dna_bench::ensure_stage_campaign_attempts(&benchmark_root, &cycle)?;
    Ok(OpsCommandOutcome::success(format!(
        "initialized {} benchmark attempts under {}\n",
        directories.len(),
        benchmark_root.display()
    )))
}

fn read_plan(path: &Path) -> Result<bijux_dna_bench::BenchmarkCampaignPlan> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_PLAN_BYTES)
        .with_context(|| format!("read benchmark campaign plan {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("parse benchmark campaign plan {}", path.display()))
}

fn source_identity(workspace: &Workspace) -> Result<(String, String)> {
    source_identity_at(&workspace.root)
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
    use super::{
        benchmark_accounting, parse_options, require_exact_options, FASTQ_PREPARATION_OPTIONS,
        FASTQ_SUBMISSION_OPTIONS,
    };

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
    fn fastq_preparation_contract_requires_complete_authorities() -> anyhow::Result<()> {
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
            "--study-root".into(),
            "/study".into(),
        ])?;
        assert!(require_exact_options(&options, FASTQ_PREPARATION_OPTIONS).is_ok());
        Ok(())
    }

    #[test]
    fn fastq_submission_contract_requires_separate_submission_time() -> anyhow::Result<()> {
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
            "--study-root".into(),
            "/study".into(),
            "--submitted-at-utc".into(),
            "2026-08-05T03:00:00Z".into(),
        ])?;
        assert!(require_exact_options(&options, FASTQ_SUBMISSION_OPTIONS).is_ok());
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
    fn fastq_admission_contract_requires_study_authority() -> anyhow::Result<()> {
        let options = parse_options(&[
            "--admitted-at-utc".into(),
            "2026-08-05T04:00:00Z".into(),
            "--benchmark-root".into(),
            "/study/results/benchmarks".into(),
            "--plan".into(),
            "/study/plan.json".into(),
            "--study-root".into(),
            "/study".into(),
        ])?;
        assert!(require_exact_options(
            &options,
            &["--admitted-at-utc", "--benchmark-root", "--plan", "--study-root"]
        )
        .is_ok());
        Ok(())
    }
}
