use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::run::run_command;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::commands::cli::{
    SlurmBundleDecryptArgs, SlurmBundleIntegrityCheck, SlurmBundleRewrapArgs,
    SlurmCampaignImportArgs, SlurmCancelArgs, SlurmCopyBackManifestArgs,
    SlurmFailureBundleExportArgs, SlurmMonitorArgs, SlurmReplayImportArgs,
    SlurmResultsPolicyCheckArgs, SlurmShareBundleArgs, SlurmSubmitCampaignArgs,
    SlurmSubmitCrossArgs, SlurmSubmitDomainArgs, SlurmSubmitStageArgs,
};
use crate::commands::hpc::{
    campaign_dry_run, decrypt_bundle, sha256_hex, sidecar_path_for, write_encrypted_bundle,
    BundleDecryptRequest, BundleWriteRequest, CampaignDryRunReport, PlannedJob,
};

const SLURM_SUBMISSION_SCHEMA_VERSION: &str = "bijux.hpc.slurm.submission.v1";
const COPY_BACK_MANIFEST_SCHEMA_VERSION: &str = "bijux.hpc.copy_back_manifest.v1";
const BUNDLE_DECRYPT_SCHEMA_VERSION: &str = "bijux.hpc.bundle.decrypt.v1";

#[derive(Debug, Clone, Serialize)]
pub struct SlurmSubmissionReport {
    pub schema_version: &'static str,
    pub mode: String,
    pub campaign_id: String,
    pub domain: String,
    pub submitted_at: String,
    pub jobs: Vec<SubmittedJob>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubmittedJob {
    pub job_name: String,
    pub stage: String,
    pub tool: String,
    pub sample: String,
    pub array_task: Option<u32>,
    pub planned_job_id: String,
    pub scheduler_job_id: String,
    pub dependency_scheduler_ids: Vec<String>,
    pub script_path: String,
    pub log_path: String,
    pub out_path: String,
    pub err_path: String,
    pub results_path: String,
    pub code_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CopyBackManifestReport {
    pub schema_version: &'static str,
    pub manifest_path: String,
    pub campaign_id: String,
    pub domain: String,
    pub suggested_copy_command: String,
    pub entries: Vec<CopyBackEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlurmCancelReport {
    pub schema_version: &'static str,
    pub mode: String,
    pub requested_job_ids: Vec<String>,
    pub cancelled_job_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlurmMonitorReport {
    pub schema_version: &'static str,
    pub campaign_id: String,
    pub domain: String,
    pub snapshot: SlurmMonitorSnapshot,
    pub jobs: Vec<SlurmMonitorEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlurmMonitorSnapshot {
    pub total_jobs: usize,
    pub jobs_with_log: usize,
    pub jobs_with_out: usize,
    pub jobs_with_err: usize,
    pub jobs_with_results_bundle: usize,
    pub jobs_with_code_bundle: usize,
    pub jobs_with_appraiser_done: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlurmMonitorEntry {
    pub planned_job_id: String,
    pub scheduler_job_id: String,
    pub stage: String,
    pub tool: String,
    pub sample: String,
    pub log_exists: bool,
    pub out_exists: bool,
    pub err_exists: bool,
    pub results_exists: bool,
    pub results_sidecar_exists: bool,
    pub results_bundle_encrypted: bool,
    pub code_exists: bool,
    pub code_sidecar_exists: bool,
    pub code_bundle_encrypted: bool,
    pub appraiser_done: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CopyBackEntry {
    pub stage: String,
    pub tool: String,
    pub sample: String,
    pub scratch_dir: String,
    pub log_path: String,
    pub out_path: String,
    pub err_path: String,
    pub results_path: String,
    pub results_sidecar_path: String,
    pub code_path: String,
    pub code_sidecar_path: String,
    pub script_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlurmDecryptReport {
    pub schema_version: &'static str,
    pub bundle_path: String,
    pub sidecar_path: String,
    pub output_path: String,
    pub output_mode: String,
    pub plaintext_sha256: String,
    pub plaintext_bytes: usize,
    pub backend: String,
    pub recipient_fingerprints: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlurmBundleIntegrityReport {
    pub schema_version: &'static str,
    pub bundle_path: String,
    pub sidecar_path: String,
    pub ok: bool,
    pub backend: String,
    pub plaintext_sha256: String,
    pub plaintext_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlurmBundleRewrapReport {
    pub schema_version: &'static str,
    pub source_bundle_path: String,
    pub output_bundle_path: String,
    pub source_sidecar_path: String,
    pub output_sidecar_path: String,
    pub plaintext_sha256: String,
    pub plaintext_bytes: usize,
    pub backend: String,
    pub recipients: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlurmReplayImportReport {
    pub schema_version: &'static str,
    pub results_bundle: String,
    pub code_bundle: String,
    pub output_root: String,
    pub results_plaintext_sha256: String,
    pub code_plaintext_sha256: String,
    pub replay_feasible: bool,
    pub completeness_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlurmCampaignImportReport {
    pub schema_version: &'static str,
    pub campaign_dir: String,
    pub output_root: String,
    pub imported_pairs: usize,
    pub failed_pairs: usize,
    pub imported: Vec<SlurmReplayImportReport>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlurmFailureBundleExportReport {
    pub schema_version: &'static str,
    pub stage: String,
    pub tool: String,
    pub sample: String,
    pub bundle_path: String,
    pub sidecar_path: String,
    pub plaintext_sha256: String,
    pub recipients: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlurmShareBundleReport {
    pub schema_version: &'static str,
    pub source_bundle_path: String,
    pub shared_bundle_path: String,
    pub shared_sidecar_path: String,
    pub plaintext_sha256: String,
    pub shared_recipients: Vec<String>,
    pub profile_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlurmResultsPolicyReport {
    pub schema_version: &'static str,
    pub results_complete: bool,
    pub code_complete: bool,
    pub appraiser_policy_ok: bool,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ShareProfile {
    pub profile_id: String,
    pub recipients: Vec<String>,
    #[serde(default = "default_share_backend")]
    pub backend: String,
}

fn default_share_backend() -> String {
    "mock-envelope-v1".to_string()
}

#[derive(Debug, Clone)]
struct SelectedJob {
    name: String,
    planned: PlannedJob,
    depends_on_names: Vec<String>,
}

#[derive(Debug, Clone)]
struct SubmissionSettings {
    mode: SubmissionMode,
    subset: SubmissionSubset,
}

#[derive(Debug, Clone)]
enum SubmissionMode {
    Mock,
    Real,
}

#[derive(Debug, Clone)]
enum SubmissionSubset {
    All,
    Stage { stage: String, tool: Option<String>, sample: Option<String> },
    Domain { domain: String },
    Cross { domains: Vec<String> },
}

fn now_timestamp_compact() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |delta| delta.as_secs());
    secs.to_string()
}

fn stage_domain(stage_id: &str) -> String {
    stage_id.split('.').next().unwrap_or("unknown").to_string()
}

fn infer_dependencies(jobs: &[SelectedJob]) -> Vec<Vec<String>> {
    let mut by_sample_last_name: BTreeMap<String, String> = BTreeMap::new();
    let mut result = Vec::with_capacity(jobs.len());
    for job in jobs {
        let mut deps = job.depends_on_names.clone();
        if let Some(previous_name) = by_sample_last_name.get(&job.planned.sample) {
            if !deps.iter().any(|dep| dep == previous_name) {
                deps.push(previous_name.clone());
            }
        }
        by_sample_last_name.insert(job.planned.sample.clone(), job.name.clone());
        result.push(deps);
    }
    result
}

fn shell_quote(value: &str) -> String {
    if value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/')) {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

fn script_path_for(log_path: &Path) -> PathBuf {
    let mut script = log_path.to_path_buf();
    let new_name = match script.file_name().and_then(|s| s.to_str()) {
        Some(name) => format!("{name}.sbatch.sh"),
        None => "job.sbatch.sh".to_string(),
    };
    script.set_file_name(new_name);
    script
}

fn ensure_parent(path: &Path) -> Result<()> {
    let Some(parent) = path.parent() else {
        return Err(anyhow!("path has no parent: {}", path.display()));
    };
    bijux_dna_infra::ensure_dir(parent).with_context(|| format!("create {}", parent.display()))
}

fn write_text(path: &Path, content: &str) -> Result<()> {
    ensure_parent(path)?;
    bijux_dna_api::v1::api::run::atomic_write_bytes(path, content.as_bytes())
        .with_context(|| format!("write {}", path.display()))
}

fn write_operator_files(
    job: &SelectedJob,
    scheduler_job_id: &str,
    submitted_at: &str,
    redaction_needles: &[String],
) -> Result<()> {
    let log_path = Path::new(&job.planned.outputs.log);
    let out_path = Path::new(&job.planned.outputs.out);
    let err_path = Path::new(&job.planned.outputs.err);

    let log = format!(
        "submitted_at={submitted_at}\njob_name={}\nscheduler_job_id={scheduler_job_id}\nstage={}\ntool={}\nsample={}\nresults={}\ncode={}\n",
        job.name,
        job.planned.stage,
        job.planned.tool,
        job.planned.sample,
        job.planned.outputs.results,
        job.planned.outputs.code
    );
    let out = "pending: scheduler output will be captured by slurm runtime wrapper\n";
    let err = "pending: scheduler stderr will be captured by slurm runtime wrapper\n";

    write_text(log_path, &redact_text(log, redaction_needles))?;
    write_text(out_path, &redact_text(out.to_string(), redaction_needles))?;
    write_text(err_path, &redact_text(err.to_string(), redaction_needles))?;
    Ok(())
}

fn git_stdout(args: &[&str]) -> Option<String> {
    let arg_vec = args.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    let output = run_command("git", &arg_vec).ok()?;
    if output.exit_code != 0 {
        return None;
    }
    Some(output.stdout.trim().to_string())
}

fn build_results_bundle(
    report: &CampaignDryRunReport,
    job: &SelectedJob,
    scheduler_job_id: &str,
    submitted_at: &str,
    redaction_needles: &[String],
) -> Result<Vec<u8>> {
    let payload = json!({
        "schema_version": "bijux.hpc.results_bundle.v1",
        "campaign": {
            "id": report.campaign_id,
            "domain": report.domain,
            "config_path": report.config_path,
            "env_file_path": report.env_file_path,
            "user_policy_path": report.user_policy_path,
        },
        "job": {
            "planned_job_id": job.planned.job_id,
            "scheduler_job_id": scheduler_job_id,
            "submitted_at": submitted_at,
            "job_name": job.name,
            "stage": job.planned.stage,
            "tool": job.planned.tool,
            "sample": job.planned.sample,
            "resource_template": job.planned.resource_template,
            "resources": job.planned.resources,
            "depends_on": job.depends_on_names,
        },
        "metrics": {
            "status": "pending_execution",
            "submission_mode": "slurm",
        },
        "artifacts": {
            "inventory": [
                {"kind": "log", "path": job.planned.outputs.log},
                {"kind": "out", "path": job.planned.outputs.out},
                {"kind": "err", "path": job.planned.outputs.err},
            ],
            "encrypted_targets": [
                {"kind": "results", "path": job.planned.outputs.results},
                {"kind": "code", "path": job.planned.outputs.code},
            ],
        },
        "reports": [{
            "kind": "submission_receipt",
            "summary": "job submitted or mocked; execution metrics pending runtime wrapper output",
        }],
        "traces": {
            "script_path": script_path_for(Path::new(&job.planned.outputs.log)).display().to_string(),
            "scheduler_job_id": scheduler_job_id,
        },
        "inventories": {
            "encryption_backend": report.security.encryption_backend,
            "recipient_fingerprints": report.security.encryption_recipients.iter().map(|recipient| {
                sha256_hex(recipient.as_bytes()).chars().take(16).collect::<String>()
            }).collect::<Vec<_>>(),
        },
        "appraiser_outputs": [{
            "name": "submission_ready",
            "status": "pending",
            "note": "appraiser jobs are tracked after runtime completion",
        }],
    });
    let text =
        serde_json::to_string_pretty(&payload).context("serialize results bundle payload")?;
    Ok(redact_text(text, redaction_needles).into_bytes())
}

fn build_code_bundle(
    report: &CampaignDryRunReport,
    job: &SelectedJob,
    script_path: &Path,
    scheduler_job_id: &str,
    submitted_at: &str,
    redaction_needles: &[String],
) -> Result<Vec<u8>> {
    let script = std::fs::read_to_string(script_path)
        .with_context(|| format!("read {}", script_path.display()))?;
    let payload = json!({
        "schema_version": "bijux.hpc.code_bundle.v1",
        "campaign": {
            "id": report.campaign_id,
            "domain": report.domain,
            "config_path": report.config_path,
            "env_file_path": report.env_file_path,
            "user_policy_path": report.user_policy_path,
            "user_policies_applied": report.user_policies_applied,
        },
        "job": {
            "planned_job_id": job.planned.job_id,
            "scheduler_job_id": scheduler_job_id,
            "submitted_at": submitted_at,
            "job_name": job.name,
            "stage": job.planned.stage,
            "tool": job.planned.tool,
            "sample": job.planned.sample,
        },
        "code_freeze": {
            "slurm_script": script,
            "effective_settings": {
                "slurm": report.resolved_slurm,
                "resources": job.planned.resources,
            },
            "config_references": {
                "campaign_config": report.config_path,
                "env_file": report.env_file_path,
                "user_policy": report.user_policy_path,
            },
            "repository_state": {
                "git_head": git_stdout(&["rev-parse", "HEAD"]),
                "git_branch": git_stdout(&["rev-parse", "--abbrev-ref", "HEAD"]),
                "git_status_porcelain": git_stdout(&["status", "--porcelain"]),
            },
            "dvc_state": {
                "available": run_command("dvc", &["--version".to_string()])
                    .map(|output| output.exit_code == 0)
                    .unwrap_or(false),
                "status_hint": "capture deferred to runtime wrapper when dvc is configured",
            },
        },
        "locks": {
            "corpus_lock": "deferred_to_prepare_corpus",
            "database_lock": "deferred_to_prepare_database",
            "image_lock": "deferred_to_prepare_apptainer",
            "tool_lock": "deferred_to_runtime_registry_capture",
        },
        "plan": {
            "depends_on": job.depends_on_names,
            "outputs": job.planned.outputs,
        },
    });
    let text = serde_json::to_string_pretty(&payload).context("serialize code bundle payload")?;
    Ok(redact_text(text, redaction_needles).into_bytes())
}

fn emit_primary_encrypted_bundles(
    report: &CampaignDryRunReport,
    job: &SelectedJob,
    script_path: &Path,
    scheduler_job_id: &str,
    submitted_at: &str,
    redaction_needles: &[String],
) -> Result<()> {
    let recipients = &report.security.encryption_recipients;
    let backend = report.security.encryption_backend.as_str();

    let results_payload =
        build_results_bundle(report, job, scheduler_job_id, submitted_at, redaction_needles)?;
    write_encrypted_bundle(&BundleWriteRequest {
        output_path: Path::new(&job.planned.outputs.results),
        bundle_kind: "results",
        campaign_id: &report.campaign_id,
        domain: &report.domain,
        stage: &job.planned.stage,
        tool: &job.planned.tool,
        sample: &job.planned.sample,
        planned_job_id: &job.planned.job_id,
        scheduler_job_id,
        submitted_at,
        backend,
        recipients,
        plaintext: &results_payload,
    })?;

    let code_payload = build_code_bundle(
        report,
        job,
        script_path,
        scheduler_job_id,
        submitted_at,
        redaction_needles,
    )?;
    write_encrypted_bundle(&BundleWriteRequest {
        output_path: Path::new(&job.planned.outputs.code),
        bundle_kind: "code",
        campaign_id: &report.campaign_id,
        domain: &report.domain,
        stage: &job.planned.stage,
        tool: &job.planned.tool,
        sample: &job.planned.sample,
        planned_job_id: &job.planned.job_id,
        scheduler_job_id,
        submitted_at,
        backend,
        recipients,
        plaintext: &code_payload,
    })?;
    Ok(())
}

fn maybe_encrypt_operator_outputs(
    report: &CampaignDryRunReport,
    job: &SelectedJob,
    scheduler_job_id: &str,
    submitted_at: &str,
    redaction_needles: &[String],
) -> Result<()> {
    if !report.security.encrypt_operator_outputs {
        return Ok(());
    }
    let recipients = &report.security.encryption_recipients;
    let backend = report.security.encryption_backend.as_str();
    for (path, kind) in [
        (job.planned.outputs.log.as_str(), "operator_log"),
        (job.planned.outputs.out.as_str(), "operator_out"),
        (job.planned.outputs.err.as_str(), "operator_err"),
    ] {
        let plaintext = std::fs::read_to_string(path).with_context(|| format!("read {path}"))?;
        let redacted = redact_text(plaintext, redaction_needles);
        write_encrypted_bundle(&BundleWriteRequest {
            output_path: Path::new(path),
            bundle_kind: kind,
            campaign_id: &report.campaign_id,
            domain: &report.domain,
            stage: &job.planned.stage,
            tool: &job.planned.tool,
            sample: &job.planned.sample,
            planned_job_id: &job.planned.job_id,
            scheduler_job_id,
            submitted_at,
            backend,
            recipients,
            plaintext: redacted.as_bytes(),
        })?;
    }
    Ok(())
}

fn build_slurm_script(
    report: &CampaignDryRunReport,
    job: &SelectedJob,
    script_path: &Path,
    dependency_scheduler_ids: &[String],
) -> String {
    let dependency_line = if dependency_scheduler_ids.is_empty() {
        String::new()
    } else {
        format!("#SBATCH --dependency=afterok:{}\n", dependency_scheduler_ids.join(":"))
    };
    let array_line =
        job.planned.array_task.map_or_else(String::new, |task| format!("#SBATCH --array={task}\n"));
    let retry_codes = report
        .resolved_slurm
        .retry_on_exit_codes
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\n#SBATCH --job-name={}\n#SBATCH --cpus-per-task={}\n#SBATCH --mem={}G\n#SBATCH --time={}\n#SBATCH --partition={}\n#SBATCH --qos={}\n{}{}\n# Campaign: {}\n# Domain: {}\n# Stage: {}\n# Tool: {}\n# Sample: {}\n# Script path: {}\n\nexport BIJUX_RUN_CONTEXT=hpc\nexport BIJUX_ARRAY_TASK=${{SLURM_ARRAY_TASK_ID:-{}}}\nexport BIJUX_SCRATCH_DIR={}\nexport BIJUX_SCRATCH_IN=$BIJUX_SCRATCH_DIR/in\nexport BIJUX_SCRATCH_OUT=$BIJUX_SCRATCH_DIR/out\nmkdir -p \"$BIJUX_SCRATCH_IN\" \"$BIJUX_SCRATCH_OUT\"\ncleanup() {{\n  rm -rf \"$BIJUX_SCRATCH_DIR\"\n}}\ntrap cleanup EXIT\n\nif [ -f {} ]; then\n  set -a\n  # shellcheck disable=SC1090\n  . {}\n  set +a\nfi\n\nretry_attempts={}\nretry_backoff_seconds={}\nretry_codes=\",{},\"\nattempt=1\nwhile true; do\n  # Placeholder command until full stage runner integration is finalized.\n  echo \\\"execute stage {} tool {} sample {} array_task=$BIJUX_ARRAY_TASK scratch=$BIJUX_SCRATCH_DIR attempt=$attempt\\\"\n  rc=0\n  if [ \"$rc\" -eq 0 ]; then\n    break\n  fi\n  if [ \"$attempt\" -ge \"$retry_attempts\" ]; then\n    exit \"$rc\"\n  fi\n  if [[ \"$retry_codes\" == *\",$rc,\"* ]]; then\n    sleep \"$retry_backoff_seconds\"\n    attempt=$((attempt + 1))\n    continue\n  fi\n  exit \"$rc\"\ndone\n",
        shell_quote(&job.name),
        job.planned.resources.cpus,
        job.planned.resources.mem_gb,
        shell_quote(&job.planned.resources.walltime),
        shell_quote(&report.resolved_slurm.partition),
        shell_quote(&report.resolved_slurm.qos),
        dependency_line,
        array_line,
        report.campaign_id,
        report.domain,
        job.planned.stage,
        job.planned.tool,
        job.planned.sample,
        script_path.display(),
        job.planned.array_task.unwrap_or(0),
        shell_quote(&job.planned.outputs.scratch_dir),
        shell_quote(&report.env_file_path),
        shell_quote(&report.env_file_path),
        report.resolved_slurm.retry_attempts,
        report.resolved_slurm.retry_backoff_seconds,
        retry_codes,
        job.planned.stage,
        job.planned.tool,
        job.planned.sample
    )
}

fn submit_with_sbatch(script_path: &Path, dependency_scheduler_ids: &[String]) -> Result<String> {
    let mut args = Vec::new();
    if !dependency_scheduler_ids.is_empty() {
        args.push(format!("--dependency=afterok:{}", dependency_scheduler_ids.join(":")));
    }
    args.push(script_path.display().to_string());
    let output = run_command("sbatch", &args)
        .with_context(|| format!("run sbatch for {}", script_path.display()))?;
    if output.exit_code != 0 {
        return Err(anyhow!("sbatch failed for {}: {}", script_path.display(), output.stderr));
    }
    let stdout = output.stdout;
    let id = stdout
        .split_whitespace()
        .find(|token| token.chars().all(|ch| ch.is_ascii_digit()))
        .ok_or_else(|| anyhow!("could not parse job id from sbatch output: {stdout}"))?;
    Ok(id.to_string())
}

fn select_jobs(
    report: &CampaignDryRunReport,
    subset: &SubmissionSubset,
) -> Result<Vec<SelectedJob>> {
    let mut selected = Vec::new();

    for planned in &report.planned_jobs {
        let include = match subset {
            SubmissionSubset::All => true,
            SubmissionSubset::Stage { stage, tool, sample } => {
                if planned.stage != *stage {
                    false
                } else if let Some(tool) = tool {
                    planned.tool == *tool
                } else if let Some(sample) = sample {
                    planned.sample == *sample
                } else {
                    true
                }
            }
            SubmissionSubset::Domain { domain } => stage_domain(&planned.stage) == *domain,
            SubmissionSubset::Cross { domains } => {
                let sd = stage_domain(&planned.stage);
                domains.iter().any(|domain| domain == &sd)
            }
        };
        if include {
            selected.push(SelectedJob {
                name: planned.job_name.clone(),
                planned: planned.clone(),
                depends_on_names: planned.depends_on.clone(),
            });
        }
    }

    if matches!(subset, SubmissionSubset::Cross { .. }) {
        let distinct_domains = selected
            .iter()
            .map(|job| stage_domain(&job.planned.stage))
            .collect::<std::collections::BTreeSet<_>>();
        if distinct_domains.len() < 2 {
            return Err(anyhow!(
                "cross-domain submission requires jobs from at least two domains; found {}",
                distinct_domains.len()
            ));
        }
    }

    if selected.is_empty() {
        return Err(anyhow!("no jobs matched submission selector"));
    }
    Ok(selected)
}

fn run_submission(
    report: CampaignDryRunReport,
    settings: SubmissionSettings,
) -> Result<SlurmSubmissionReport> {
    let selected = select_jobs(&report, &settings.subset)?;
    let dependency_name_graph = infer_dependencies(&selected);
    let redaction_needles = redaction_needles(&report)?;

    let submitted_at = now_timestamp_compact();
    let mut name_to_scheduler_id: BTreeMap<String, String> = BTreeMap::new();
    let mut jobs_out = Vec::new();

    for (index, selected_job) in selected.iter().enumerate() {
        let dependency_scheduler_ids = dependency_name_graph[index]
            .iter()
            .filter_map(|name| name_to_scheduler_id.get(name).cloned())
            .collect::<Vec<_>>();

        let script_path = script_path_for(Path::new(&selected_job.planned.outputs.log));
        let script =
            build_slurm_script(&report, selected_job, &script_path, &dependency_scheduler_ids);
        write_text(&script_path, &script)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path)
                .with_context(|| format!("stat {}", script_path.display()))?
                .permissions();
            perms.set_mode(0o750);
            std::fs::set_permissions(&script_path, perms)
                .with_context(|| format!("chmod {}", script_path.display()))?;
        }

        let scheduler_job_id = match settings.mode {
            SubmissionMode::Mock => format!("mock-{:04}", index + 1),
            SubmissionMode::Real => submit_with_sbatch(&script_path, &dependency_scheduler_ids)?,
        };

        write_operator_files(selected_job, &scheduler_job_id, &submitted_at, &redaction_needles)?;
        emit_primary_encrypted_bundles(
            &report,
            selected_job,
            &script_path,
            &scheduler_job_id,
            &submitted_at,
            &redaction_needles,
        )?;
        maybe_encrypt_operator_outputs(
            &report,
            selected_job,
            &scheduler_job_id,
            &submitted_at,
            &redaction_needles,
        )?;

        name_to_scheduler_id.insert(selected_job.name.clone(), scheduler_job_id.clone());
        jobs_out.push(SubmittedJob {
            job_name: selected_job.name.clone(),
            stage: selected_job.planned.stage.clone(),
            tool: selected_job.planned.tool.clone(),
            sample: selected_job.planned.sample.clone(),
            array_task: selected_job.planned.array_task,
            planned_job_id: selected_job.planned.job_id.clone(),
            scheduler_job_id,
            dependency_scheduler_ids,
            script_path: script_path.display().to_string(),
            log_path: selected_job.planned.outputs.log.clone(),
            out_path: selected_job.planned.outputs.out.clone(),
            err_path: selected_job.planned.outputs.err.clone(),
            results_path: selected_job.planned.outputs.results.clone(),
            code_path: selected_job.planned.outputs.code.clone(),
        });
    }

    Ok(SlurmSubmissionReport {
        schema_version: SLURM_SUBMISSION_SCHEMA_VERSION,
        mode: match settings.mode {
            SubmissionMode::Mock => "mock".to_string(),
            SubmissionMode::Real => "real".to_string(),
        },
        campaign_id: report.campaign_id,
        domain: report.domain,
        submitted_at,
        jobs: jobs_out,
    })
}

pub fn submit_stage_benchmark(args: &SlurmSubmitStageArgs) -> Result<SlurmSubmissionReport> {
    let report =
        campaign_dry_run(&args.config, args.env_file.as_deref(), args.user_policies.as_deref())?;
    run_submission(
        report,
        SubmissionSettings {
            mode: if args.mock_submit { SubmissionMode::Mock } else { SubmissionMode::Real },
            subset: SubmissionSubset::Stage {
                stage: args.stage.clone(),
                tool: args.tool.clone(),
                sample: args.sample.clone(),
            },
        },
    )
}

pub fn submit_domain_benchmark(args: &SlurmSubmitDomainArgs) -> Result<SlurmSubmissionReport> {
    let report =
        campaign_dry_run(&args.config, args.env_file.as_deref(), args.user_policies.as_deref())?;
    run_submission(
        report,
        SubmissionSettings {
            mode: if args.mock_submit { SubmissionMode::Mock } else { SubmissionMode::Real },
            subset: SubmissionSubset::Domain { domain: args.domain.clone() },
        },
    )
}

pub fn submit_cross_benchmark(args: &SlurmSubmitCrossArgs) -> Result<SlurmSubmissionReport> {
    let report =
        campaign_dry_run(&args.config, args.env_file.as_deref(), args.user_policies.as_deref())?;
    let domains = args
        .domains
        .as_deref()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .filter(|values| !values.is_empty())
        .unwrap_or_else(|| {
            report
                .planned_jobs
                .iter()
                .map(|job| stage_domain(&job.stage))
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        });
    run_submission(
        report,
        SubmissionSettings {
            mode: if args.mock_submit { SubmissionMode::Mock } else { SubmissionMode::Real },
            subset: SubmissionSubset::Cross { domains },
        },
    )
}

pub fn submit_campaign(args: &SlurmSubmitCampaignArgs) -> Result<SlurmSubmissionReport> {
    let report =
        campaign_dry_run(&args.config, args.env_file.as_deref(), args.user_policies.as_deref())?;
    run_submission(
        report,
        SubmissionSettings {
            mode: if args.mock_submit { SubmissionMode::Mock } else { SubmissionMode::Real },
            subset: SubmissionSubset::All,
        },
    )
}

fn collect_cancel_job_ids(args: &SlurmCancelArgs) -> Result<Vec<String>> {
    let mut ids = args
        .job_id
        .iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if let Some(manifest_path) = &args.manifest {
        let raw = std::fs::read_to_string(manifest_path)
            .with_context(|| format!("read {}", manifest_path.display()))?;
        let value: serde_json::Value = serde_json::from_str(&raw)
            .with_context(|| format!("parse {}", manifest_path.display()))?;
        let manifest_ids = value
            .get("jobs")
            .and_then(|rows| rows.as_array())
            .into_iter()
            .flatten()
            .filter_map(|row| row.get("scheduler_job_id").and_then(|id| id.as_str()))
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        ids.extend(manifest_ids);
    }
    ids.sort();
    ids.dedup();
    if ids.is_empty() {
        return Err(anyhow!("cancel requires --job-id values or --manifest containing jobs"));
    }
    Ok(ids)
}

pub fn cancel_jobs(args: &SlurmCancelArgs) -> Result<SlurmCancelReport> {
    let requested_job_ids = collect_cancel_job_ids(args)?;
    if args.mock_cancel {
        return Ok(SlurmCancelReport {
            schema_version: SLURM_SUBMISSION_SCHEMA_VERSION,
            mode: "mock".to_string(),
            requested_job_ids: requested_job_ids.clone(),
            cancelled_job_ids: requested_job_ids,
        });
    }
    let output = run_command("scancel", &requested_job_ids).context("run scancel")?;
    if output.exit_code != 0 {
        return Err(anyhow!("scancel failed: {}", output.stderr.trim()));
    }
    Ok(SlurmCancelReport {
        schema_version: SLURM_SUBMISSION_SCHEMA_VERSION,
        mode: "real".to_string(),
        requested_job_ids: requested_job_ids.clone(),
        cancelled_job_ids: requested_job_ids,
    })
}

fn is_encrypted_bundle_marker(path: &Path) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    let preview_len = bytes.len().min(512);
    let preview = String::from_utf8_lossy(&bytes[..preview_len]);
    preview.contains("\"schema_version\": \"bijux.hpc.bundle.")
}

#[derive(Debug, Deserialize)]
struct SubmissionManifestJobRow {
    planned_job_id: Option<String>,
    scheduler_job_id: Option<String>,
    log_path: Option<String>,
    out_path: Option<String>,
    err_path: Option<String>,
    results_path: Option<String>,
    code_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SubmissionManifestFile {
    jobs: Vec<SubmissionManifestJobRow>,
}

fn submission_jobs_from_manifest(
    path: &Path,
) -> Result<BTreeMap<String, SubmissionManifestJobRow>> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let manifest: SubmissionManifestFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    Ok(manifest
        .jobs
        .into_iter()
        .filter_map(|row| row.planned_job_id.clone().map(|planned_id| (planned_id, row)))
        .collect())
}

pub fn monitor_campaign(args: &SlurmMonitorArgs) -> Result<SlurmMonitorReport> {
    let report =
        campaign_dry_run(&args.config, args.env_file.as_deref(), args.user_policies.as_deref())?;
    let submitted_jobs = if let Some(path) = &args.submission_manifest {
        submission_jobs_from_manifest(path)?
    } else {
        BTreeMap::new()
    };
    let mut entries = Vec::new();
    for job in &report.planned_jobs {
        let submitted_job = submitted_jobs.get(&job.job_id);
        let log_path =
            submitted_job.and_then(|row| row.log_path.as_deref()).unwrap_or(&job.outputs.log);
        let out_path =
            submitted_job.and_then(|row| row.out_path.as_deref()).unwrap_or(&job.outputs.out);
        let err_path =
            submitted_job.and_then(|row| row.err_path.as_deref()).unwrap_or(&job.outputs.err);
        let results_path = submitted_job
            .and_then(|row| row.results_path.as_deref())
            .unwrap_or(&job.outputs.results);
        let code_path =
            submitted_job.and_then(|row| row.code_path.as_deref()).unwrap_or(&job.outputs.code);
        let log_exists = Path::new(log_path).is_file();
        let out_exists = Path::new(out_path).is_file();
        let err_exists = Path::new(err_path).is_file();
        let results_exists = Path::new(results_path).is_file();
        let code_exists = Path::new(code_path).is_file();
        let results_sidecar_exists = sidecar_path_for(Path::new(results_path)).is_file();
        let code_sidecar_exists = sidecar_path_for(Path::new(code_path)).is_file();
        let results_bundle_encrypted =
            results_exists && is_encrypted_bundle_marker(Path::new(results_path));
        let code_bundle_encrypted = code_exists && is_encrypted_bundle_marker(Path::new(code_path));
        let appraiser_done = PathBuf::from(format!("{results_path}.appraiser.done")).is_file();
        entries.push(SlurmMonitorEntry {
            planned_job_id: job.job_id.clone(),
            scheduler_job_id: submitted_job
                .and_then(|row| row.scheduler_job_id.clone())
                .unwrap_or_else(|| "<unknown>".to_string()),
            stage: job.stage.clone(),
            tool: job.tool.clone(),
            sample: job.sample.clone(),
            log_exists,
            out_exists,
            err_exists,
            results_exists,
            results_sidecar_exists,
            results_bundle_encrypted,
            code_exists,
            code_sidecar_exists,
            code_bundle_encrypted,
            appraiser_done,
        });
    }
    let snapshot = SlurmMonitorSnapshot {
        total_jobs: entries.len(),
        jobs_with_log: entries.iter().filter(|row| row.log_exists).count(),
        jobs_with_out: entries.iter().filter(|row| row.out_exists).count(),
        jobs_with_err: entries.iter().filter(|row| row.err_exists).count(),
        jobs_with_results_bundle: entries
            .iter()
            .filter(|row| {
                row.results_exists && row.results_sidecar_exists && row.results_bundle_encrypted
            })
            .count(),
        jobs_with_code_bundle: entries
            .iter()
            .filter(|row| row.code_exists && row.code_sidecar_exists && row.code_bundle_encrypted)
            .count(),
        jobs_with_appraiser_done: entries.iter().filter(|row| row.appraiser_done).count(),
    };
    Ok(SlurmMonitorReport {
        schema_version: SLURM_SUBMISSION_SCHEMA_VERSION,
        campaign_id: report.campaign_id,
        domain: report.domain,
        snapshot,
        jobs: entries,
    })
}

pub fn write_copy_back_manifest(
    args: &SlurmCopyBackManifestArgs,
) -> Result<CopyBackManifestReport> {
    let report =
        campaign_dry_run(&args.config, args.env_file.as_deref(), args.user_policies.as_deref())?;
    let manifest_path = args.out.clone().unwrap_or_else(|| {
        Path::new(&report.config_path).parent().map_or_else(
            || PathBuf::from("artifacts/slurm_copy_back_manifest.json"),
            |parent| parent.join("copy-back-manifest.json"),
        )
    });

    let entries = report
        .planned_jobs
        .iter()
        .map(|job| CopyBackEntry {
            stage: job.stage.clone(),
            tool: job.tool.clone(),
            sample: job.sample.clone(),
            scratch_dir: job.outputs.scratch_dir.clone(),
            log_path: job.outputs.log.clone(),
            out_path: job.outputs.out.clone(),
            err_path: job.outputs.err.clone(),
            results_path: job.outputs.results.clone(),
            results_sidecar_path: sidecar_path_for(Path::new(&job.outputs.results))
                .display()
                .to_string(),
            code_path: job.outputs.code.clone(),
            code_sidecar_path: sidecar_path_for(Path::new(&job.outputs.code)).display().to_string(),
            script_path: script_path_for(Path::new(&job.outputs.log)).display().to_string(),
        })
        .collect::<Vec<_>>();

    let suggested_copy_command = if let Some(first) = entries.first() {
        format!(
            "rsync -av {} {} {} {} {} {} {} <destination_dir>/",
            shell_quote(&first.log_path),
            shell_quote(&first.out_path),
            shell_quote(&first.err_path),
            shell_quote(&first.results_path),
            shell_quote(&first.results_sidecar_path),
            shell_quote(&first.code_path),
            shell_quote(&first.code_sidecar_path)
        )
    } else {
        "rsync -av <source> <destination_dir>/".to_string()
    };

    let manifest = CopyBackManifestReport {
        schema_version: COPY_BACK_MANIFEST_SCHEMA_VERSION,
        manifest_path: manifest_path.display().to_string(),
        campaign_id: report.campaign_id,
        domain: report.domain,
        suggested_copy_command,
        entries,
    };

    let payload = serde_json::to_vec_pretty(&manifest).context("serialize copy-back manifest")?;
    ensure_parent(&manifest_path)?;
    bijux_dna_api::v1::api::run::atomic_write_bytes(&manifest_path, &payload)
        .with_context(|| format!("write {}", manifest_path.display()))?;

    Ok(manifest)
}

fn ensure_private_directory(path: &Path) -> Result<()> {
    bijux_dna_infra::ensure_dir(path).with_context(|| format!("create {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)
            .with_context(|| format!("stat {}", path.display()))?
            .permissions();
        perms.set_mode(0o700);
        std::fs::set_permissions(path, perms)
            .with_context(|| format!("chmod {}", path.display()))?;
    }
    Ok(())
}

fn decrypted_output_path(bundle_path: &Path, out_dir: &Path) -> PathBuf {
    let mut file = bundle_path
        .file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| "bundle".to_string(), ToOwned::to_owned);
    file.push_str(".decrypted.json");
    out_dir.join(file)
}

fn bundle_output_path(bundle_path: &Path, out_dir: &Path, suffix: &str) -> PathBuf {
    let mut file = bundle_path
        .file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| "bundle".to_string(), ToOwned::to_owned);
    file.push_str(suffix);
    out_dir.join(file)
}

fn normalize_recipients(values: &[String]) -> Vec<String> {
    values.iter().map(|value| value.trim().to_string()).filter(|value| !value.is_empty()).collect()
}

fn unsafe_destination(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(path)
            .with_context(|| format!("stat {}", path.display()))?
            .permissions()
            .mode();
        if (mode & 0o007) != 0 || (mode & 0o070) != 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

fn ensure_safe_private_directory(path: &Path, allow_unsafe_destination: bool) -> Result<()> {
    if unsafe_destination(path)? && !allow_unsafe_destination {
        return Err(anyhow!(
            "refuse unsafe decrypt destination {}; pass --allow-unsafe-destination to policy",
            path.display()
        ));
    }
    ensure_private_directory(path)
}

fn load_env_map_for_redaction(env_path: &Path) -> Result<BTreeMap<String, String>> {
    if !env_path.exists() {
        return Ok(BTreeMap::new());
    }
    let raw = std::fs::read_to_string(env_path)
        .with_context(|| format!("read {}", env_path.display()))?;
    let mut map = BTreeMap::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.splitn(2, '=');
        let Some(key) = parts.next() else { continue };
        let Some(value) = parts.next() else { continue };
        map.insert(key.trim().to_string(), value.trim().trim_matches('"').to_string());
    }
    Ok(map)
}

fn redaction_needles(report: &CampaignDryRunReport) -> Result<Vec<String>> {
    let env_map = load_env_map_for_redaction(Path::new(&report.env_file_path))?;
    let mut needles = Vec::new();
    for (key, value) in env_map {
        if value.is_empty() {
            continue;
        }
        let uppercase = key.to_ascii_uppercase();
        let by_name = uppercase.contains("SECRET")
            || uppercase.contains("TOKEN")
            || uppercase.contains("PASSWORD")
            || uppercase.contains("KEY")
            || uppercase.contains("ACCOUNT")
            || uppercase.contains("PROJECT");
        let explicitly = report.security.redacted_env_keys.iter().any(|k| k == &key);
        if (by_name || explicitly) && value.len() >= 4 {
            needles.push(value);
        }
    }
    needles.sort();
    needles.dedup();
    Ok(needles)
}

fn redact_text(mut text: String, needles: &[String]) -> String {
    for needle in needles {
        if needle.is_empty() {
            continue;
        }
        text = text.replace(needle, "<redacted>");
    }
    text
}

fn required_result_paths(_payload: &serde_json::Value) -> Vec<&'static str> {
    vec![
        "metrics",
        "artifacts.inventory",
        "reports",
        "appraiser_outputs",
        "job.stage",
        "job.tool",
        "job.sample",
    ]
}

fn required_code_paths(payload: &serde_json::Value) -> Vec<&'static str> {
    let _ = payload;
    vec![
        "code_freeze.repository_state",
        "code_freeze.config_references",
        "code_freeze.slurm_script",
        "locks.corpus_lock",
        "locks.database_lock",
        "locks.image_lock",
        "plan.outputs",
    ]
}

fn has_json_path(root: &serde_json::Value, path: &str) -> bool {
    let mut node = root;
    for part in path.split('.') {
        let Some(next) = node.get(part) else {
            return false;
        };
        node = next;
    }
    true
}

fn validate_results_payload(payload: &serde_json::Value) -> Vec<String> {
    required_result_paths(payload)
        .into_iter()
        .filter(|path| !has_json_path(payload, path))
        .map(|path| format!("missing results field `{path}`"))
        .collect()
}

fn validate_code_payload(payload: &serde_json::Value) -> Vec<String> {
    required_code_paths(payload)
        .into_iter()
        .filter(|path| !has_json_path(payload, path))
        .map(|path| format!("missing code field `{path}`"))
        .collect()
}

fn appraiser_output_policy_issues(results_payload: &serde_json::Value) -> Vec<String> {
    let mut issues = Vec::new();
    if !has_json_path(results_payload, "appraiser_outputs") {
        issues.push("results payload lacks `appraiser_outputs`".to_string());
    }
    if let Some(entries) = results_payload.get("artifacts").and_then(|v| v.get("inventory")) {
        if entries.as_array().is_some_and(|rows| {
            rows.iter().any(|row| {
                row.get("kind") == Some(&serde_json::Value::String("appraiser_output".to_string()))
            })
        }) {
            issues.push("appraiser output must not be listed as plaintext artifact".to_string());
        }
    }
    issues
}

fn normalized_sidecar_for_bundle(
    bundle_path: &Path,
    original_sidecar: &Path,
    temp_dir: &Path,
) -> Result<PathBuf> {
    let raw = std::fs::read(original_sidecar)
        .with_context(|| format!("read {}", original_sidecar.display()))?;
    let mut sidecar: serde_json::Value =
        serde_json::from_slice(&raw).context("parse sidecar for normalization")?;
    let current = sidecar.get("ciphertext_path").and_then(|v| v.as_str()).unwrap_or("");
    if Path::new(current) == bundle_path {
        return Ok(original_sidecar.to_path_buf());
    }
    sidecar["ciphertext_path"] =
        serde_json::Value::String(bundle_path.as_os_str().to_string_lossy().to_string());
    let normalized =
        temp_dir.join(original_sidecar.file_name().and_then(|name| name.to_str()).map_or_else(
            || "normalized.sidecar.json".to_string(),
            |name| format!("{name}.normalized"),
        ));
    let payload = serde_json::to_vec_pretty(&sidecar).context("serialize normalized sidecar")?;
    bijux_dna_api::v1::api::run::atomic_write_bytes(&normalized, &payload)
        .with_context(|| format!("write {}", normalized.display()))?;
    Ok(normalized)
}

pub fn decrypt_bundle_to_local(args: &SlurmBundleDecryptArgs) -> Result<SlurmDecryptReport> {
    ensure_safe_private_directory(&args.out_dir, args.allow_unsafe_destination)?;
    let sidecar_path = args.sidecar.clone().unwrap_or_else(|| sidecar_path_for(&args.bundle));
    let (sidecar, plaintext) = decrypt_bundle(&BundleDecryptRequest {
        bundle_path: &args.bundle,
        sidecar_path: Some(&sidecar_path),
        identity_files: &args.identity_file,
    })?;
    let output_path = decrypted_output_path(&args.bundle, &args.out_dir);
    bijux_dna_api::v1::api::run::atomic_write_bytes(&output_path, &plaintext)
        .with_context(|| format!("write {}", output_path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&output_path)
            .with_context(|| format!("stat {}", output_path.display()))?
            .permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&output_path, perms)
            .with_context(|| format!("chmod {}", output_path.display()))?;
    }

    Ok(SlurmDecryptReport {
        schema_version: BUNDLE_DECRYPT_SCHEMA_VERSION,
        bundle_path: args.bundle.display().to_string(),
        sidecar_path: sidecar_path.display().to_string(),
        output_path: output_path.display().to_string(),
        output_mode: "file".to_string(),
        plaintext_sha256: sidecar.plaintext_sha256,
        plaintext_bytes: plaintext.len(),
        backend: sidecar.backend,
        recipient_fingerprints: sidecar.recipient_fingerprints,
    })
}

pub fn verify_bundle_integrity(
    args: &SlurmBundleIntegrityCheck,
) -> Result<SlurmBundleIntegrityReport> {
    let sidecar_path = args.sidecar.clone().unwrap_or_else(|| sidecar_path_for(&args.bundle));
    let (sidecar, plaintext) = decrypt_bundle(&BundleDecryptRequest {
        bundle_path: &args.bundle,
        sidecar_path: Some(&sidecar_path),
        identity_files: &args.identity_file,
    })?;
    Ok(SlurmBundleIntegrityReport {
        schema_version: BUNDLE_DECRYPT_SCHEMA_VERSION,
        bundle_path: args.bundle.display().to_string(),
        sidecar_path: sidecar_path.display().to_string(),
        ok: true,
        backend: sidecar.backend,
        plaintext_sha256: sidecar.plaintext_sha256,
        plaintext_bytes: plaintext.len(),
    })
}

pub fn rewrap_bundle(args: &SlurmBundleRewrapArgs) -> Result<SlurmBundleRewrapReport> {
    let sidecar_path = args.sidecar.clone().unwrap_or_else(|| sidecar_path_for(&args.bundle));
    let (sidecar, plaintext) = decrypt_bundle(&BundleDecryptRequest {
        bundle_path: &args.bundle,
        sidecar_path: Some(&sidecar_path),
        identity_files: &args.identity_file,
    })?;
    let recipients = normalize_recipients(&args.recipient);
    if recipients.is_empty() {
        return Err(anyhow!("rewrap requires at least one --recipient"));
    }
    let output_bundle = args.out_bundle.clone().unwrap_or_else(|| args.bundle.clone());
    let output = write_encrypted_bundle(&BundleWriteRequest {
        output_path: &output_bundle,
        bundle_kind: &sidecar.bundle_kind,
        campaign_id: &sidecar.campaign_id,
        domain: &sidecar.domain,
        stage: &sidecar.stage,
        tool: &sidecar.tool,
        sample: &sidecar.sample,
        planned_job_id: &sidecar.planned_job_id,
        scheduler_job_id: &sidecar.scheduler_job_id,
        submitted_at: &sidecar.submitted_at,
        backend: &sidecar.backend,
        recipients: &recipients,
        plaintext: &plaintext,
    })?;
    Ok(SlurmBundleRewrapReport {
        schema_version: BUNDLE_DECRYPT_SCHEMA_VERSION,
        source_bundle_path: args.bundle.display().to_string(),
        output_bundle_path: output_bundle.display().to_string(),
        source_sidecar_path: sidecar_path.display().to_string(),
        output_sidecar_path: sidecar_path_for(&output_bundle).display().to_string(),
        plaintext_sha256: output.plaintext_sha256,
        plaintext_bytes: output.plaintext_bytes,
        backend: output.backend,
        recipients: output.recipients,
    })
}

pub fn import_encrypted_replay(args: &SlurmReplayImportArgs) -> Result<SlurmReplayImportReport> {
    ensure_safe_private_directory(&args.out_dir, args.allow_unsafe_destination)?;
    let results_sidecar =
        args.results_sidecar.clone().unwrap_or_else(|| sidecar_path_for(&args.results_bundle));
    let code_sidecar =
        args.code_sidecar.clone().unwrap_or_else(|| sidecar_path_for(&args.code_bundle));

    let (results_meta, results_plaintext) = decrypt_bundle(&BundleDecryptRequest {
        bundle_path: &args.results_bundle,
        sidecar_path: Some(&results_sidecar),
        identity_files: &args.identity_file,
    })?;
    let (code_meta, code_plaintext) = decrypt_bundle(&BundleDecryptRequest {
        bundle_path: &args.code_bundle,
        sidecar_path: Some(&code_sidecar),
        identity_files: &args.identity_file,
    })?;

    let results_out =
        bundle_output_path(&args.results_bundle, &args.out_dir, ".replay.results.json");
    let code_out = bundle_output_path(&args.code_bundle, &args.out_dir, ".replay.code.json");
    bijux_dna_api::v1::api::run::atomic_write_bytes(&results_out, &results_plaintext)?;
    bijux_dna_api::v1::api::run::atomic_write_bytes(&code_out, &code_plaintext)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [&results_out, &code_out] {
            let mut perms = std::fs::metadata(path)
                .with_context(|| format!("stat {}", path.display()))?
                .permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(path, perms)
                .with_context(|| format!("chmod {}", path.display()))?;
        }
    }

    let results_json: serde_json::Value =
        serde_json::from_slice(&results_plaintext).context("parse results replay payload")?;
    let code_json: serde_json::Value =
        serde_json::from_slice(&code_plaintext).context("parse code replay payload")?;
    let mut checks = Vec::new();
    checks.extend(validate_results_payload(&results_json));
    checks.extend(validate_code_payload(&code_json));
    let replay_feasible = checks.is_empty();

    let report = SlurmReplayImportReport {
        schema_version: BUNDLE_DECRYPT_SCHEMA_VERSION,
        results_bundle: args.results_bundle.display().to_string(),
        code_bundle: args.code_bundle.display().to_string(),
        output_root: args.out_dir.display().to_string(),
        results_plaintext_sha256: results_meta.plaintext_sha256,
        code_plaintext_sha256: code_meta.plaintext_sha256,
        replay_feasible,
        completeness_checks: checks,
    };
    let manifest_path = args.out_dir.join("import-replay-report.json");
    let payload = serde_json::to_vec_pretty(&report).context("serialize replay import report")?;
    bijux_dna_api::v1::api::run::atomic_write_bytes(&manifest_path, &payload)
        .with_context(|| format!("write {}", manifest_path.display()))?;
    Ok(report)
}

pub fn import_encrypted_campaign(
    args: &SlurmCampaignImportArgs,
) -> Result<SlurmCampaignImportReport> {
    ensure_safe_private_directory(&args.out_dir, args.allow_unsafe_destination)?;
    if !args.campaign_dir.is_dir() {
        return Err(anyhow!(
            "campaign import requires directory; got {}",
            args.campaign_dir.display()
        ));
    }

    let mut results_bundles = Vec::new();
    for entry in std::fs::read_dir(&args.campaign_dir)
        .with_context(|| format!("read {}", args.campaign_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("results") {
            results_bundles.push(path);
        }
    }
    results_bundles.sort();

    let mut imported = Vec::new();
    let mut errors = Vec::new();
    for results in results_bundles {
        let Some(results_name) = results.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let code_name = results_name
            .strip_suffix(".results")
            .map_or_else(|| format!("{results_name}.code"), |prefix| format!("{prefix}.code"));
        let code_path = results.with_file_name(code_name);
        if !code_path.is_file() {
            errors.push(format!("missing code bundle for {}", results.display()));
            continue;
        }

        let replay_out = args.out_dir.join(
            results
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map_or_else(|| "campaign-row".to_string(), ToOwned::to_owned),
        );
        ensure_private_directory(&replay_out)?;
        let normalized_meta_dir = replay_out.join("sidecars");
        ensure_private_directory(&normalized_meta_dir)?;
        let normalized_results_sidecar = normalized_sidecar_for_bundle(
            &results,
            &sidecar_path_for(&results),
            &normalized_meta_dir,
        )?;
        let normalized_code_sidecar = normalized_sidecar_for_bundle(
            &code_path,
            &sidecar_path_for(&code_path),
            &normalized_meta_dir,
        )?;
        let report = import_encrypted_replay(&SlurmReplayImportArgs {
            results_bundle: results.clone(),
            results_sidecar: Some(normalized_results_sidecar),
            code_bundle: code_path,
            code_sidecar: Some(normalized_code_sidecar),
            out_dir: replay_out,
            identity_file: args.identity_file.clone(),
            allow_unsafe_destination: args.allow_unsafe_destination,
            json: false,
        });
        match report {
            Ok(value) => imported.push(value),
            Err(error) => errors.push(format!("{}: {error}", results.display())),
        }
    }

    let failed_pairs = errors.len();
    let report = SlurmCampaignImportReport {
        schema_version: BUNDLE_DECRYPT_SCHEMA_VERSION,
        campaign_dir: args.campaign_dir.display().to_string(),
        output_root: args.out_dir.display().to_string(),
        imported_pairs: imported.len(),
        failed_pairs,
        imported,
        errors,
    };
    let manifest_path = args.out_dir.join("import-campaign-report.json");
    let payload = serde_json::to_vec_pretty(&report).context("serialize campaign import report")?;
    bijux_dna_api::v1::api::run::atomic_write_bytes(&manifest_path, &payload)
        .with_context(|| format!("write {}", manifest_path.display()))?;
    Ok(report)
}

pub fn export_failure_bundle(
    args: &SlurmFailureBundleExportArgs,
) -> Result<SlurmFailureBundleExportReport> {
    let report =
        campaign_dry_run(&args.config, args.env_file.as_deref(), args.user_policies.as_deref())?;
    let Some(job) = report
        .planned_jobs
        .iter()
        .find(|job| job.stage == args.stage && job.tool == args.tool && job.sample == args.sample)
    else {
        return Err(anyhow!(
            "no job matched stage={} tool={} sample={}",
            args.stage,
            args.tool,
            args.sample
        ));
    };
    ensure_parent(&args.out_dir.join("placeholder"))?;
    let recipients = normalize_recipients(&args.recipient);
    if recipients.is_empty() {
        return Err(anyhow!("failure export requires at least one --recipient"));
    }
    let payload = json!({
        "schema_version": "bijux.hpc.failure_bundle.v1",
        "campaign_id": report.campaign_id,
        "domain": report.domain,
        "failure_row": {
            "stage": job.stage,
            "tool": job.tool,
            "sample": job.sample,
            "job_name": job.job_name,
            "planned_job_id": job.job_id,
        },
        "minimal_context": {
            "resources": job.resources,
            "resource_template": job.resource_template,
            "outputs": job.outputs,
        },
    });
    let plaintext =
        serde_json::to_vec_pretty(&payload).context("serialize failure bundle plaintext")?;
    let file_name = format!("{}-{}-{}.failure", args.stage, args.tool, args.sample);
    let bundle_path = args.out_dir.join(file_name);
    let sidecar = write_encrypted_bundle(&BundleWriteRequest {
        output_path: &bundle_path,
        bundle_kind: "failure_export",
        campaign_id: &report.campaign_id,
        domain: &report.domain,
        stage: &job.stage,
        tool: &job.tool,
        sample: &job.sample,
        planned_job_id: &job.job_id,
        scheduler_job_id: "unavailable",
        submitted_at: "unavailable",
        backend: &args.backend,
        recipients: &recipients,
        plaintext: &plaintext,
    })?;
    let report = SlurmFailureBundleExportReport {
        schema_version: BUNDLE_DECRYPT_SCHEMA_VERSION,
        stage: job.stage.clone(),
        tool: job.tool.clone(),
        sample: job.sample.clone(),
        bundle_path: bundle_path.display().to_string(),
        sidecar_path: sidecar_path_for(&bundle_path).display().to_string(),
        plaintext_sha256: sidecar.plaintext_sha256,
        recipients: sidecar.recipients,
    };
    let manifest_path = args.out_dir.join("failure-export-report.json");
    let payload = serde_json::to_vec_pretty(&report).context("serialize failure export report")?;
    bijux_dna_api::v1::api::run::atomic_write_bytes(&manifest_path, &payload)
        .with_context(|| format!("write {}", manifest_path.display()))?;
    Ok(report)
}

pub fn share_bundle_with_profile(args: &SlurmShareBundleArgs) -> Result<SlurmShareBundleReport> {
    let sidecar_path = args.sidecar.clone().unwrap_or_else(|| sidecar_path_for(&args.bundle));
    let profile_raw = std::fs::read_to_string(&args.profile)
        .with_context(|| format!("read {}", args.profile.display()))?;
    let profile: ShareProfile = toml::from_str(&profile_raw).context("parse share profile")?;
    if profile.recipients.is_empty() {
        return Err(anyhow!("share profile has no recipients"));
    }
    ensure_parent(&args.out_dir.join("placeholder"))?;
    let (source_meta, plaintext) = decrypt_bundle(&BundleDecryptRequest {
        bundle_path: &args.bundle,
        sidecar_path: Some(&sidecar_path),
        identity_files: &args.identity_file,
    })?;

    let shared_bundle = args.out_dir.join(
        args.bundle
            .file_name()
            .and_then(|name| name.to_str())
            .map_or_else(|| "shared.bundle".to_string(), |name| format!("{name}.shared")),
    );
    let sidecar = write_encrypted_bundle(&BundleWriteRequest {
        output_path: &shared_bundle,
        bundle_kind: &source_meta.bundle_kind,
        campaign_id: &source_meta.campaign_id,
        domain: &source_meta.domain,
        stage: &source_meta.stage,
        tool: &source_meta.tool,
        sample: &source_meta.sample,
        planned_job_id: &source_meta.planned_job_id,
        scheduler_job_id: &source_meta.scheduler_job_id,
        submitted_at: &source_meta.submitted_at,
        backend: &profile.backend,
        recipients: &profile.recipients,
        plaintext: &plaintext,
    })?;

    let mut redacted_sidecar = sidecar.clone();
    redacted_sidecar.stage = "<redacted>".to_string();
    redacted_sidecar.tool = "<redacted>".to_string();
    redacted_sidecar.sample = "<redacted>".to_string();
    redacted_sidecar.campaign_id = "<redacted>".to_string();
    let redacted_path = sidecar_path_for(&shared_bundle);
    let payload =
        serde_json::to_vec_pretty(&redacted_sidecar).context("serialize redacted sidecar")?;
    bijux_dna_api::v1::api::run::atomic_write_bytes(&redacted_path, &payload)
        .with_context(|| format!("write {}", redacted_path.display()))?;

    let report = SlurmShareBundleReport {
        schema_version: BUNDLE_DECRYPT_SCHEMA_VERSION,
        source_bundle_path: args.bundle.display().to_string(),
        shared_bundle_path: shared_bundle.display().to_string(),
        shared_sidecar_path: redacted_path.display().to_string(),
        plaintext_sha256: sidecar.plaintext_sha256,
        shared_recipients: profile.recipients,
        profile_id: profile.profile_id,
    };
    let manifest_path = args.out_dir.join("share-bundle-report.json");
    let payload = serde_json::to_vec_pretty(&report).context("serialize share bundle report")?;
    bijux_dna_api::v1::api::run::atomic_write_bytes(&manifest_path, &payload)
        .with_context(|| format!("write {}", manifest_path.display()))?;
    Ok(report)
}

pub fn verify_results_policy(
    args: &SlurmResultsPolicyCheckArgs,
) -> Result<SlurmResultsPolicyReport> {
    let results_sidecar =
        args.results_sidecar.clone().unwrap_or_else(|| sidecar_path_for(&args.results_bundle));
    let code_sidecar =
        args.code_sidecar.clone().unwrap_or_else(|| sidecar_path_for(&args.code_bundle));
    let (_, results_plaintext) = decrypt_bundle(&BundleDecryptRequest {
        bundle_path: &args.results_bundle,
        sidecar_path: Some(&results_sidecar),
        identity_files: &args.identity_file,
    })?;
    let (_, code_plaintext) = decrypt_bundle(&BundleDecryptRequest {
        bundle_path: &args.code_bundle,
        sidecar_path: Some(&code_sidecar),
        identity_files: &args.identity_file,
    })?;
    let results_json: serde_json::Value =
        serde_json::from_slice(&results_plaintext).context("parse results bundle json")?;
    let code_json: serde_json::Value =
        serde_json::from_slice(&code_plaintext).context("parse code bundle json")?;

    let mut issues = validate_results_payload(&results_json);
    issues.extend(validate_code_payload(&code_json));
    issues.extend(appraiser_output_policy_issues(&results_json));
    let results_complete = validate_results_payload(&results_json).is_empty();
    let code_complete = validate_code_payload(&code_json).is_empty();
    let appraiser_policy_ok = appraiser_output_policy_issues(&results_json).is_empty();

    Ok(SlurmResultsPolicyReport {
        schema_version: BUNDLE_DECRYPT_SCHEMA_VERSION,
        results_complete,
        code_complete,
        appraiser_policy_ok,
        issues,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{
        cancel_jobs, decrypt_bundle_to_local, export_failure_bundle, import_encrypted_campaign,
        import_encrypted_replay, monitor_campaign, rewrap_bundle, share_bundle_with_profile,
        submit_campaign, submit_cross_benchmark, submit_domain_benchmark, submit_stage_benchmark,
        verify_bundle_integrity, verify_results_policy, write_copy_back_manifest,
    };
    use crate::commands::cli::{
        SlurmBundleDecryptArgs, SlurmBundleIntegrityCheck, SlurmBundleRewrapArgs,
        SlurmCampaignImportArgs, SlurmCancelArgs, SlurmCopyBackManifestArgs,
        SlurmFailureBundleExportArgs, SlurmMonitorArgs, SlurmReplayImportArgs,
        SlurmResultsPolicyCheckArgs, SlurmShareBundleArgs, SlurmSubmitCampaignArgs,
        SlurmSubmitCrossArgs, SlurmSubmitDomainArgs, SlurmSubmitStageArgs,
    };
    use crate::commands::hpc::{BundleDecryptRequest, BundleWriteRequest};

    fn write_campaign_with_security(
        root: &std::path::Path,
        encryption_backend: &str,
        encrypt_operator_outputs: bool,
    ) -> std::path::PathBuf {
        for name in [
            "corpora",
            "databases",
            "images",
            "scratch",
            "logs",
            "results",
            "code",
            "imports",
            "baselines",
        ] {
            bijux_dna_infra::ensure_dir(root.join(name)).expect("create dir");
        }
        let env_path = root.join("campaign.env");
        bijux_dna_infra::write_bytes(&env_path, "BIJUX_SLURM_ACCOUNT=a\nBIJUX_SLURM_PROJECT=p\n")
            .expect("write env");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&env_path).expect("env metadata").permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&env_path, perms).expect("set env perms");
        }

        let config_path = root.join("campaign.toml");
        let config = format!(
            r#"
[campaign]
id = "mini"
domain = "cross"

[layout]
corpora_root = "{root}/corpora"
databases_root = "{root}/databases"
images_root = "{root}/images"
scratch_root = "{root}/scratch"
logs_root = "{root}/logs"
encrypted_results_root = "{root}/results"
encrypted_code_root = "{root}/code"
appraiser_imports_root = "{root}/imports"
baselines_root = "{root}/baselines"

[slurm]
site_profile = "generic"

[resources]
default = "standard"

[resources.templates.standard]
cpus = 1
mem_gb = 1
walltime = "00:05:00"
scratch_gb = 1

[security]
encryption_backend = "{encryption_backend}"
encryption_recipients = ["alice"]
encrypt_operator_outputs = {encrypt_operator_outputs}
env_file = "{root}/campaign.env"

[[jobs]]
name = "fastq_validate_sample1"
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"

[[jobs]]
name = "bam_sort_sample1"
stage = "bam.sort"
tool = "samtools"
sample = "sample-1"
depends_on = ["fastq_validate_sample1"]

[[jobs]]
name = "vcf_validate_sample2"
stage = "vcf.validate"
tool = "bcftools"
sample = "sample-2"
"#,
            root = root.display(),
            encryption_backend = encryption_backend,
            encrypt_operator_outputs = encrypt_operator_outputs
        );
        bijux_dna_infra::write_bytes(&config_path, config).expect("write config");
        config_path
    }

    fn write_campaign(root: &std::path::Path) -> std::path::PathBuf {
        write_campaign_with_security(root, "mock-envelope-v1", false)
    }

    #[test]
    fn submit_stage_benchmark_filters_rows() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_stage_benchmark(&SlurmSubmitStageArgs {
            config,
            env_file: None,
            user_policies: None,
            stage: "fastq.validate_reads".to_string(),
            tool: None,
            sample: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit stage");
        assert_eq!(report.jobs.len(), 1);
        assert_eq!(report.jobs[0].stage, "fastq.validate_reads");
    }

    #[test]
    fn submit_stage_benchmark_emits_array_directive_when_array_task_is_set() {
        let root = tempfile::tempdir().expect("tempdir");
        for name in [
            "corpora",
            "databases",
            "images",
            "scratch",
            "logs",
            "results",
            "code",
            "imports",
            "baselines",
        ] {
            bijux_dna_infra::ensure_dir(root.path().join(name)).expect("create dir");
        }
        let env_path = root.path().join("campaign.env");
        bijux_dna_infra::write_bytes(&env_path, "BIJUX_SLURM_ACCOUNT=a\nBIJUX_SLURM_PROJECT=p\n")
            .expect("write env");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&env_path).expect("env metadata").permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&env_path, perms).expect("set env perms");
        }

        let config_path = root.path().join("campaign-array.toml");
        let config = format!(
            r#"
[campaign]
id = "mini-array"
domain = "fastq"

[layout]
corpora_root = "{root}/corpora"
databases_root = "{root}/databases"
images_root = "{root}/images"
scratch_root = "{root}/scratch"
logs_root = "{root}/logs"
encrypted_results_root = "{root}/results"
encrypted_code_root = "{root}/code"
appraiser_imports_root = "{root}/imports"
baselines_root = "{root}/baselines"

[slurm]
site_profile = "generic"

[resources]
default = "standard"

[resources.templates.standard]
cpus = 1
mem_gb = 1
walltime = "00:05:00"
scratch_gb = 1

[security]
encryption_backend = "mock-envelope-v1"
encryption_recipients = ["alice"]
env_file = "{root}/campaign.env"

[[jobs]]
name = "fastq_validate_array"
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-array"
array_task = 7
"#,
            root = root.path().display()
        );
        bijux_dna_infra::write_bytes(&config_path, config).expect("write config");

        let report = submit_stage_benchmark(&SlurmSubmitStageArgs {
            config: config_path,
            env_file: None,
            user_policies: None,
            stage: "fastq.validate_reads".to_string(),
            tool: None,
            sample: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit stage");
        assert_eq!(report.jobs.len(), 1);
        assert_eq!(report.jobs[0].array_task, Some(7));
        let script = std::fs::read_to_string(&report.jobs[0].script_path).expect("read script");
        assert!(script.contains("#SBATCH --array=7"));
        assert!(script.contains("export BIJUX_ARRAY_TASK=${SLURM_ARRAY_TASK_ID:-7}"));
    }

    #[test]
    fn submit_domain_benchmark_filters_domain() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_domain_benchmark(&SlurmSubmitDomainArgs {
            config,
            env_file: None,
            user_policies: None,
            domain: "bam".to_string(),
            mock_submit: true,
            json: false,
        })
        .expect("submit domain");
        assert_eq!(report.jobs.len(), 1);
        assert_eq!(report.jobs[0].stage, "bam.sort");
    }

    #[test]
    fn submit_cross_benchmark_requires_multiple_domains() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_cross_benchmark(&SlurmSubmitCrossArgs {
            config,
            env_file: None,
            user_policies: None,
            domains: Some("fastq,bam".to_string()),
            mock_submit: true,
            json: false,
        })
        .expect("submit cross");
        assert_eq!(report.jobs.len(), 2);
    }

    #[test]
    fn submit_campaign_writes_operator_files() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config,
            env_file: None,
            user_policies: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit campaign");
        assert_eq!(report.jobs.len(), 3);
        assert_eq!(report.jobs[1].dependency_scheduler_ids, vec!["mock-0001".to_string()]);
        for job in report.jobs {
            assert!(std::path::Path::new(&job.log_path).is_file());
            assert!(std::path::Path::new(&job.out_path).is_file());
            assert!(std::path::Path::new(&job.err_path).is_file());
            assert!(std::path::Path::new(&job.script_path).is_file());
        }
    }

    #[test]
    fn cancel_jobs_accepts_manifest_and_job_ids_in_mock_mode() {
        let root = tempfile::tempdir().expect("tempdir");
        let manifest_path = root.path().join("submission.json");
        let manifest = serde_json::json!({
            "schema_version": "bijux.hpc.slurm.submission.v1",
            "jobs": [
                { "scheduler_job_id": "1234" },
                { "scheduler_job_id": "2234" }
            ]
        });
        bijux_dna_infra::write_bytes(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
        )
        .expect("write manifest");

        let report = cancel_jobs(&SlurmCancelArgs {
            job_id: vec!["3234".to_string()],
            manifest: Some(manifest_path),
            mock_cancel: true,
            json: false,
        })
        .expect("cancel mock");
        assert_eq!(report.mode, "mock");
        assert_eq!(report.requested_job_ids, vec!["1234", "2234", "3234"]);
        assert_eq!(report.cancelled_job_ids, report.requested_job_ids);
    }

    #[test]
    fn monitor_campaign_reports_bundle_and_sidecar_snapshots() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let submission = submit_campaign(&SlurmSubmitCampaignArgs {
            config: config.clone(),
            env_file: None,
            user_policies: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit campaign");
        let submission_manifest = root.path().join("submission-report.json");
        bijux_dna_infra::write_bytes(
            &submission_manifest,
            serde_json::to_vec_pretty(&submission).expect("serialize submission"),
        )
        .expect("write submission report");
        let first_results = std::path::PathBuf::from(&submission.jobs[0].results_path);
        bijux_dna_infra::write_bytes(
            format!("{}.appraiser.done", first_results.display()),
            b"done",
        )
        .expect("write appraiser marker");

        let report = monitor_campaign(&SlurmMonitorArgs {
            config,
            env_file: None,
            user_policies: None,
            submission_manifest: Some(submission_manifest),
            json: false,
        })
        .expect("monitor");
        assert_eq!(report.snapshot.total_jobs, 3);
        assert_eq!(report.snapshot.jobs_with_log, 3);
        assert_eq!(report.snapshot.jobs_with_out, 3);
        assert_eq!(report.snapshot.jobs_with_err, 3);
        assert_eq!(report.snapshot.jobs_with_results_bundle, 3);
        assert_eq!(report.snapshot.jobs_with_code_bundle, 3);
        assert_eq!(report.snapshot.jobs_with_appraiser_done, 1);
        assert!(report.jobs.iter().all(|job| job.scheduler_job_id != "<unknown>"));
    }

    #[test]
    fn submit_campaign_scripts_include_strict_mode_and_dependency_directives() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config,
            env_file: None,
            user_policies: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit campaign");

        let first_script =
            std::fs::read_to_string(&report.jobs[0].script_path).expect("read script 1");
        assert!(first_script.contains("set -euo pipefail"));
        assert!(first_script.contains("export BIJUX_SCRATCH_DIR="));
        assert!(first_script.contains("mkdir -p \"$BIJUX_SCRATCH_IN\" \"$BIJUX_SCRATCH_OUT\""));
        assert!(first_script.contains("trap cleanup EXIT"));
        assert!(!first_script.contains("--dependency=afterok"));

        let second_script =
            std::fs::read_to_string(&report.jobs[1].script_path).expect("read script 2");
        assert!(second_script.contains("set -euo pipefail"));
        assert!(second_script.contains("#SBATCH --dependency=afterok:mock-0001"));
    }

    #[test]
    fn submit_campaign_scripts_include_retry_policy_when_configured() {
        let root = tempfile::tempdir().expect("tempdir");
        for name in [
            "corpora",
            "databases",
            "images",
            "scratch",
            "logs",
            "results",
            "code",
            "imports",
            "baselines",
        ] {
            bijux_dna_infra::ensure_dir(root.path().join(name)).expect("create dir");
        }
        let env_path = root.path().join("campaign.env");
        bijux_dna_infra::write_bytes(&env_path, "BIJUX_SLURM_ACCOUNT=a\nBIJUX_SLURM_PROJECT=p\n")
            .expect("write env");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&env_path).expect("env metadata").permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&env_path, perms).expect("set env perms");
        }

        let config_path = root.path().join("campaign-retry.toml");
        let config = format!(
            r#"
[campaign]
id = "mini-retry"
domain = "fastq"

[layout]
corpora_root = "{root}/corpora"
databases_root = "{root}/databases"
images_root = "{root}/images"
scratch_root = "{root}/scratch"
logs_root = "{root}/logs"
encrypted_results_root = "{root}/results"
encrypted_code_root = "{root}/code"
appraiser_imports_root = "{root}/imports"
baselines_root = "{root}/baselines"

[slurm]
site_profile = "generic"
retry_attempts = 3
retry_backoff_seconds = 15
retry_on_exit_codes = [1, 2, 137]

[resources]
default = "standard"

[resources.templates.standard]
cpus = 1
mem_gb = 1
walltime = "00:05:00"
scratch_gb = 1

[security]
encryption_backend = "mock-envelope-v1"
encryption_recipients = ["alice"]
env_file = "{root}/campaign.env"

[[jobs]]
name = "fastq_validate_retry"
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-retry"
"#,
            root = root.path().display()
        );
        bijux_dna_infra::write_bytes(&config_path, config).expect("write config");
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config: config_path,
            env_file: None,
            user_policies: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit campaign");
        let script = std::fs::read_to_string(&report.jobs[0].script_path).expect("read script");
        assert!(script.contains("retry_attempts=3"));
        assert!(script.contains("retry_backoff_seconds=15"));
        assert!(script.contains("retry_codes=\",1,2,137,\""));
        assert!(script.contains("while true; do"));
    }

    #[test]
    fn copy_back_manifest_writes_expected_entries() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let out = root.path().join("copy-back.json");
        let report = write_copy_back_manifest(&SlurmCopyBackManifestArgs {
            config,
            env_file: None,
            user_policies: None,
            out: Some(out.clone()),
            json: false,
        })
        .expect("write manifest");
        assert_eq!(report.entries.len(), 3);
        assert!(report.suggested_copy_command.starts_with("rsync -av "));
        assert!(report.entries.iter().all(|entry| !entry.script_path.is_empty()));
        assert!(report.entries.iter().all(|entry| !entry.scratch_dir.is_empty()));
        assert!(out.is_file());
    }

    #[test]
    fn submit_campaign_writes_encrypted_results_and_code_with_sidecars() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config,
            env_file: None,
            user_policies: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit campaign");

        for job in report.jobs {
            let results_path = std::path::PathBuf::from(&job.results_path);
            let code_path = std::path::PathBuf::from(&job.code_path);
            assert!(results_path.is_file());
            assert!(code_path.is_file());
            assert!(super::sidecar_path_for(&results_path).is_file());
            assert!(super::sidecar_path_for(&code_path).is_file());
        }
    }

    #[test]
    fn submit_campaign_keeps_operator_outputs_readable_by_default() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config,
            env_file: None,
            user_policies: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit campaign");

        let first = &report.jobs[0];
        let log = std::fs::read_to_string(&first.log_path).expect("read log");
        let out = std::fs::read_to_string(&first.out_path).expect("read out");
        let err = std::fs::read_to_string(&first.err_path).expect("read err");
        assert!(log.contains("scheduler_job_id=mock-0001"));
        assert!(out.contains("pending"));
        assert!(err.contains("pending"));
        assert!(!super::sidecar_path_for(std::path::Path::new(&first.log_path)).exists());
        assert!(!super::sidecar_path_for(std::path::Path::new(&first.out_path)).exists());
        assert!(!super::sidecar_path_for(std::path::Path::new(&first.err_path)).exists());
    }

    #[test]
    fn submit_campaign_encrypts_operator_outputs_when_enabled() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign_with_security(root.path(), "mock-envelope-v1", true);
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config,
            env_file: None,
            user_policies: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit campaign");

        let first = &report.jobs[0];
        let log = std::fs::read_to_string(&first.log_path).expect("read encrypted log");
        assert!(log.contains("\"schema_version\": \"bijux.hpc.bundle.mock_envelope.v1\""));
        assert!(super::sidecar_path_for(std::path::Path::new(&first.log_path)).is_file());
        assert!(super::sidecar_path_for(std::path::Path::new(&first.out_path)).is_file());
        assert!(super::sidecar_path_for(std::path::Path::new(&first.err_path)).is_file());
    }

    #[test]
    fn decrypt_bundle_to_local_recovers_plaintext_bundle() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config,
            env_file: None,
            user_policies: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit campaign");
        let first = &report.jobs[0];
        let out_dir = root.path().join("decrypted");
        let decrypt_report = decrypt_bundle_to_local(&SlurmBundleDecryptArgs {
            bundle: std::path::PathBuf::from(&first.results_path),
            sidecar: None,
            out_dir: out_dir.clone(),
            identity_file: Vec::new(),
            allow_unsafe_destination: false,
            json: false,
        })
        .expect("decrypt bundle");
        assert!(std::path::Path::new(&decrypt_report.output_path).is_file());
        let plaintext = std::fs::read_to_string(std::path::Path::new(&decrypt_report.output_path))
            .expect("read");
        assert!(plaintext.contains("\"schema_version\": \"bijux.hpc.results_bundle.v1\""));
    }

    #[test]
    fn verify_bundle_integrity_reports_ok_for_valid_bundle() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config,
            env_file: None,
            user_policies: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit campaign");
        let first = &report.jobs[0];
        let check = verify_bundle_integrity(&SlurmBundleIntegrityCheck {
            bundle: std::path::PathBuf::from(&first.code_path),
            sidecar: None,
            identity_file: Vec::new(),
            json: false,
        })
        .expect("verify integrity");
        assert!(check.ok);
    }

    #[test]
    fn submit_campaign_fails_without_partial_plaintext_bundles_on_backend_error() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign_with_security(root.path(), "unsupported-backend", false);
        let err = submit_campaign(&SlurmSubmitCampaignArgs {
            config,
            env_file: None,
            user_policies: None,
            mock_submit: true,
            json: false,
        })
        .expect_err("must fail for unsupported backend");
        assert!(err.to_string().contains("unsupported encryption backend"));
        let leaked_results = root.path().join("results");
        let leaked_code = root.path().join("code");
        let results_entries = std::fs::read_dir(leaked_results).expect("results dir").count();
        let code_entries = std::fs::read_dir(leaked_code).expect("code dir").count();
        assert_eq!(results_entries, 0);
        assert_eq!(code_entries, 0);
    }

    #[test]
    fn decrypt_bundle_refuses_world_readable_destination_by_default() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config,
            env_file: None,
            user_policies: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit campaign");
        let first = &report.jobs[0];
        let out_dir = root.path().join("unsafe-decrypt");
        bijux_dna_infra::ensure_dir(&out_dir).expect("create out dir");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&out_dir).expect("metadata").permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&out_dir, perms).expect("chmod");
        }

        let err = decrypt_bundle_to_local(&SlurmBundleDecryptArgs {
            bundle: std::path::PathBuf::from(&first.results_path),
            sidecar: None,
            out_dir: out_dir.clone(),
            identity_file: Vec::new(),
            allow_unsafe_destination: false,
            json: false,
        })
        .expect_err("must reject unsafe destination");
        assert!(err.to_string().contains("refuse unsafe decrypt destination"));

        let ok = decrypt_bundle_to_local(&SlurmBundleDecryptArgs {
            bundle: std::path::PathBuf::from(&first.results_path),
            sidecar: None,
            out_dir,
            identity_file: Vec::new(),
            allow_unsafe_destination: true,
            json: false,
        })
        .expect("allow unsafe destination");
        assert!(std::path::Path::new(&ok.output_path).is_file());
    }

    #[test]
    fn rewrap_bundle_preserves_plaintext_identity() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config,
            env_file: None,
            user_policies: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit campaign");
        let first = &report.jobs[0];
        let original = verify_bundle_integrity(&SlurmBundleIntegrityCheck {
            bundle: std::path::PathBuf::from(&first.results_path),
            sidecar: None,
            identity_file: Vec::new(),
            json: false,
        })
        .expect("verify original");
        let out_bundle = root.path().join("rewrapped.results");
        let rewrapped = rewrap_bundle(&SlurmBundleRewrapArgs {
            bundle: std::path::PathBuf::from(&first.results_path),
            sidecar: None,
            identity_file: Vec::new(),
            recipient: vec!["charlie".to_string()],
            out_bundle: Some(out_bundle.clone()),
            json: false,
        })
        .expect("rewrap");
        assert_eq!(rewrapped.plaintext_sha256, original.plaintext_sha256);
        assert!(out_bundle.is_file());
    }

    #[test]
    fn import_replay_reports_feasible_for_complete_pair() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config,
            env_file: None,
            user_policies: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit campaign");
        let first = &report.jobs[0];
        let imported = import_encrypted_replay(&SlurmReplayImportArgs {
            results_bundle: std::path::PathBuf::from(&first.results_path),
            results_sidecar: None,
            code_bundle: std::path::PathBuf::from(&first.code_path),
            code_sidecar: None,
            out_dir: root.path().join("replay"),
            identity_file: Vec::new(),
            allow_unsafe_destination: false,
            json: false,
        })
        .expect("import replay");
        assert!(imported.replay_feasible);
        assert!(imported.completeness_checks.is_empty());
        assert!(root.path().join("replay/import-replay-report.json").is_file());
    }

    #[test]
    fn import_campaign_ingests_results_code_pairs() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config,
            env_file: None,
            user_policies: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit campaign");
        let campaign_dir = root.path().join("campaign-copy");
        bijux_dna_infra::ensure_dir(&campaign_dir).expect("mkdir");
        let first = &report.jobs[0];
        for path in [&first.results_path, &first.code_path] {
            let src = std::path::Path::new(path);
            let dst = campaign_dir.join(src.file_name().expect("filename"));
            std::fs::copy(src, &dst).expect("copy bundle");
            let src_side = super::sidecar_path_for(src);
            let dst_side = campaign_dir.join(src_side.file_name().expect("side name"));
            std::fs::copy(src_side, dst_side).expect("copy sidecar");
        }
        let imported = import_encrypted_campaign(&SlurmCampaignImportArgs {
            campaign_dir: campaign_dir.clone(),
            out_dir: root.path().join("campaign-import"),
            identity_file: Vec::new(),
            allow_unsafe_destination: false,
            json: false,
        })
        .expect("import campaign");
        assert_eq!(imported.imported_pairs, 1, "errors={:?}", imported.errors);
        assert_eq!(imported.failed_pairs, 0);
        assert!(root.path().join("campaign-import/import-campaign-report.json").is_file());
    }

    #[test]
    fn export_failure_bundle_writes_minimal_encrypted_bundle() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let out = root.path().join("failure-export");
        let report = export_failure_bundle(&SlurmFailureBundleExportArgs {
            config,
            env_file: None,
            user_policies: None,
            stage: "fastq.validate_reads".to_string(),
            tool: "seqkit_v2".to_string(),
            sample: "sample-1".to_string(),
            out_dir: out,
            recipient: vec!["alice".to_string()],
            backend: "mock-envelope-v1".to_string(),
            json: false,
        })
        .expect("export failure");
        assert!(std::path::Path::new(&report.bundle_path).is_file());
        assert!(std::path::Path::new(&report.sidecar_path).is_file());
        assert!(root.path().join("failure-export/failure-export-report.json").is_file());
    }

    #[test]
    fn share_bundle_profile_reencrypts_and_redacts_public_sidecar() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config,
            env_file: None,
            user_policies: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit campaign");
        let profile = root.path().join("collab-profile.toml");
        bijux_dna_infra::write_bytes(
            &profile,
            "profile_id = \"collab-a\"\nbackend = \"mock-envelope-v1\"\nrecipients = [\"team-a\"]\n",
        )
        .expect("write profile");
        let shared = share_bundle_with_profile(&SlurmShareBundleArgs {
            bundle: std::path::PathBuf::from(&report.jobs[0].results_path),
            sidecar: None,
            identity_file: Vec::new(),
            profile,
            out_dir: root.path().join("shared"),
            json: false,
        })
        .expect("share bundle");
        let sidecar = std::fs::read_to_string(&shared.shared_sidecar_path).expect("read sidecar");
        assert!(sidecar.contains("\"campaign_id\": \"<redacted>\""));
        assert!(sidecar.contains("\"stage\": \"<redacted>\""));
        assert!(root.path().join("shared/share-bundle-report.json").is_file());
    }

    #[test]
    fn verify_results_policy_detects_incomplete_results_and_code() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config,
            env_file: None,
            user_policies: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit campaign");
        let first = &report.jobs[0];

        let (results_sidecar, results_plaintext) = super::decrypt_bundle(&BundleDecryptRequest {
            bundle_path: std::path::Path::new(&first.results_path),
            sidecar_path: None,
            identity_files: &[],
        })
        .expect("decrypt results");
        let (code_sidecar, code_plaintext) = super::decrypt_bundle(&BundleDecryptRequest {
            bundle_path: std::path::Path::new(&first.code_path),
            sidecar_path: None,
            identity_files: &[],
        })
        .expect("decrypt code");
        let mut results_json: serde_json::Value =
            serde_json::from_slice(&results_plaintext).expect("parse results");
        let mut code_json: serde_json::Value =
            serde_json::from_slice(&code_plaintext).expect("parse code");
        results_json.as_object_mut().expect("obj").remove("appraiser_outputs");
        code_json
            .get_mut("code_freeze")
            .and_then(|v| v.as_object_mut())
            .expect("code_freeze")
            .remove("repository_state");

        let bad_results = root.path().join("bad.results");
        let bad_code = root.path().join("bad.code");
        let recipients = vec!["alice".to_string()];
        super::write_encrypted_bundle(&BundleWriteRequest {
            output_path: &bad_results,
            bundle_kind: &results_sidecar.bundle_kind,
            campaign_id: &results_sidecar.campaign_id,
            domain: &results_sidecar.domain,
            stage: &results_sidecar.stage,
            tool: &results_sidecar.tool,
            sample: &results_sidecar.sample,
            planned_job_id: &results_sidecar.planned_job_id,
            scheduler_job_id: &results_sidecar.scheduler_job_id,
            submitted_at: &results_sidecar.submitted_at,
            backend: "mock-envelope-v1",
            recipients: &recipients,
            plaintext: &serde_json::to_vec_pretty(&results_json).expect("serialize"),
        })
        .expect("write bad results");
        super::write_encrypted_bundle(&BundleWriteRequest {
            output_path: &bad_code,
            bundle_kind: &code_sidecar.bundle_kind,
            campaign_id: &code_sidecar.campaign_id,
            domain: &code_sidecar.domain,
            stage: &code_sidecar.stage,
            tool: &code_sidecar.tool,
            sample: &code_sidecar.sample,
            planned_job_id: &code_sidecar.planned_job_id,
            scheduler_job_id: &code_sidecar.scheduler_job_id,
            submitted_at: &code_sidecar.submitted_at,
            backend: "mock-envelope-v1",
            recipients: &recipients,
            plaintext: &serde_json::to_vec_pretty(&code_json).expect("serialize"),
        })
        .expect("write bad code");

        let policy = verify_results_policy(&SlurmResultsPolicyCheckArgs {
            results_bundle: bad_results,
            results_sidecar: None,
            code_bundle: bad_code,
            code_sidecar: None,
            identity_file: Vec::new(),
            json: false,
        })
        .expect("verify policy");
        assert!(!policy.results_complete);
        assert!(!policy.code_complete);
        assert!(!policy.appraiser_policy_ok);
    }

    #[test]
    fn submit_campaign_redacts_seeded_secret_values_in_operator_outputs() {
        let root = tempfile::tempdir().expect("tempdir");
        for name in [
            "corpora",
            "databases",
            "images",
            "scratch",
            "logs",
            "results",
            "code",
            "imports",
            "baselines",
        ] {
            bijux_dna_infra::ensure_dir(root.path().join(name)).expect("create dir");
        }
        let secret = "SENSITIVE_TOKEN_12345";
        let env_path = root.path().join("campaign.env");
        bijux_dna_infra::write_bytes(
            &env_path,
            format!("BIJUX_SLURM_ACCOUNT=a\nBIJUX_SLURM_PROJECT=p\nBIJUX_API_TOKEN={secret}\n"),
        )
        .expect("write env");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&env_path).expect("env metadata").permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&env_path, perms).expect("set env perms");
        }
        let config_path = root.path().join("campaign.toml");
        let config = format!(
            r#"
[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "{root}/corpora"
databases_root = "{root}/databases"
images_root = "{root}/images"
scratch_root = "{root}/scratch"
logs_root = "{root}/logs"
encrypted_results_root = "{root}/results"
encrypted_code_root = "{root}/code"
appraiser_imports_root = "{root}/imports"
baselines_root = "{root}/baselines"

[slurm]
site_profile = "generic"

[resources]
default = "standard"

[resources.templates.standard]
cpus = 1
mem_gb = 1
walltime = "00:05:00"
scratch_gb = 1

[security]
encryption_backend = "mock-envelope-v1"
encryption_recipients = ["alice"]
env_file = "{root}/campaign.env"

[[jobs]]
name = "fastq_validate_secret"
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "{secret}"
"#,
            root = root.path().display(),
            secret = secret
        );
        bijux_dna_infra::write_bytes(&config_path, config).expect("write config");
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config: config_path,
            env_file: None,
            user_policies: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit campaign");
        let log = std::fs::read_to_string(&report.jobs[0].log_path).expect("read log");
        assert!(!log.contains(secret));
        assert!(log.contains("<redacted>"));
    }
}
