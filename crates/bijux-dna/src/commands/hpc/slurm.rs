use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::json;

use crate::commands::cli::{
    SlurmBundleDecryptArgs, SlurmBundleIntegrityCheck,
    SlurmCopyBackManifestArgs, SlurmSubmitCampaignArgs, SlurmSubmitCrossArgs,
    SlurmSubmitDomainArgs, SlurmSubmitStageArgs,
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
pub struct CopyBackEntry {
    pub stage: String,
    pub tool: String,
    pub sample: String,
    pub log_path: String,
    pub out_path: String,
    pub err_path: String,
    pub results_path: String,
    pub code_path: String,
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

    write_text(log_path, &log)?;
    write_text(out_path, out)?;
    write_text(err_path, err)?;
    Ok(())
}

fn git_stdout(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn build_results_bundle(
    report: &CampaignDryRunReport,
    job: &SelectedJob,
    scheduler_job_id: &str,
    submitted_at: &str,
) -> Result<Vec<u8>> {
    let payload = json!({
        "schema_version": "bijux.hpc.results_bundle.v1",
        "campaign": {
            "id": report.campaign_id,
            "domain": report.domain,
            "config_path": report.config_path,
            "env_file_path": report.env_file_path,
            "user_override_path": report.user_override_path,
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
    serde_json::to_vec_pretty(&payload).context("serialize results bundle payload")
}

fn build_code_bundle(
    report: &CampaignDryRunReport,
    job: &SelectedJob,
    script_path: &Path,
    scheduler_job_id: &str,
    submitted_at: &str,
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
            "user_override_path": report.user_override_path,
            "user_overrides_applied": report.user_overrides_applied,
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
                "user_override": report.user_override_path,
            },
            "repository_state": {
                "git_head": git_stdout(&["rev-parse", "HEAD"]),
                "git_branch": git_stdout(&["rev-parse", "--abbrev-ref", "HEAD"]),
                "git_status_porcelain": git_stdout(&["status", "--porcelain"]),
            },
            "dvc_state": {
                "available": Command::new("dvc").arg("--version").output().is_ok(),
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
    serde_json::to_vec_pretty(&payload).context("serialize code bundle payload")
}

fn emit_primary_encrypted_bundles(
    report: &CampaignDryRunReport,
    job: &SelectedJob,
    script_path: &Path,
    scheduler_job_id: &str,
    submitted_at: &str,
) -> Result<()> {
    let recipients = &report.security.encryption_recipients;
    let backend = report.security.encryption_backend.as_str();

    let results_payload = build_results_bundle(report, job, scheduler_job_id, submitted_at)?;
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

    let code_payload = build_code_bundle(report, job, script_path, scheduler_job_id, submitted_at)?;
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
        let plaintext = std::fs::read(path).with_context(|| format!("read {path}"))?;
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
            plaintext: &plaintext,
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
        "".to_string()
    } else {
        format!("#SBATCH --dependency=afterok:{}\n", dependency_scheduler_ids.join(":"))
    };

    format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\n#SBATCH --job-name={}\n#SBATCH --cpus-per-task={}\n#SBATCH --mem={}G\n#SBATCH --time={}\n#SBATCH --partition={}\n#SBATCH --qos={}\n{}\n# Campaign: {}\n# Domain: {}\n# Stage: {}\n# Tool: {}\n# Sample: {}\n# Script path: {}\n\nexport BIJUX_RUN_CONTEXT=hpc\n\nif [ -f {} ]; then\n  set -a\n  # shellcheck disable=SC1090\n  . {}\n  set +a\nfi\n\n# Placeholder command until full stage runner integration is finalized.\necho \\\"execute stage {} tool {} sample {}\\\"\n",
        shell_quote(&job.name),
        job.planned.resources.cpus,
        job.planned.resources.mem_gb,
        shell_quote(&job.planned.resources.walltime),
        shell_quote(&report.resolved_slurm.partition),
        shell_quote(&report.resolved_slurm.qos),
        dependency_line,
        report.campaign_id,
        report.domain,
        job.planned.stage,
        job.planned.tool,
        job.planned.sample,
        script_path.display(),
        shell_quote(&report.env_file_path),
        shell_quote(&report.env_file_path),
        job.planned.stage,
        job.planned.tool,
        job.planned.sample
    )
}

fn submit_with_sbatch(script_path: &Path, dependency_scheduler_ids: &[String]) -> Result<String> {
    let mut command = Command::new("sbatch");
    if !dependency_scheduler_ids.is_empty() {
        command.arg(format!("--dependency=afterok:{}", dependency_scheduler_ids.join(":")));
    }
    command.arg(script_path);
    let output =
        command.output().with_context(|| format!("run sbatch for {}", script_path.display()))?;
    if !output.status.success() {
        return Err(anyhow!(
            "sbatch failed for {}: {}",
            script_path.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
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

        write_operator_files(selected_job, &scheduler_job_id, &submitted_at)?;
        emit_primary_encrypted_bundles(
            &report,
            selected_job,
            &script_path,
            &scheduler_job_id,
            &submitted_at,
        )?;
        maybe_encrypt_operator_outputs(&report, selected_job, &scheduler_job_id, &submitted_at)?;

        name_to_scheduler_id.insert(selected_job.name.clone(), scheduler_job_id.clone());
        jobs_out.push(SubmittedJob {
            job_name: selected_job.name.clone(),
            stage: selected_job.planned.stage.clone(),
            tool: selected_job.planned.tool.clone(),
            sample: selected_job.planned.sample.clone(),
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
        campaign_dry_run(&args.config, args.env_file.as_deref(), args.user_overrides.as_deref())?;
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
        campaign_dry_run(&args.config, args.env_file.as_deref(), args.user_overrides.as_deref())?;
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
        campaign_dry_run(&args.config, args.env_file.as_deref(), args.user_overrides.as_deref())?;
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
        campaign_dry_run(&args.config, args.env_file.as_deref(), args.user_overrides.as_deref())?;
    run_submission(
        report,
        SubmissionSettings {
            mode: if args.mock_submit { SubmissionMode::Mock } else { SubmissionMode::Real },
            subset: SubmissionSubset::All,
        },
    )
}

pub fn write_copy_back_manifest(
    args: &SlurmCopyBackManifestArgs,
) -> Result<CopyBackManifestReport> {
    let report =
        campaign_dry_run(&args.config, args.env_file.as_deref(), args.user_overrides.as_deref())?;
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
            log_path: job.outputs.log.clone(),
            out_path: job.outputs.out.clone(),
            err_path: job.outputs.err.clone(),
            results_path: job.outputs.results.clone(),
            code_path: job.outputs.code.clone(),
            script_path: script_path_for(Path::new(&job.outputs.log)).display().to_string(),
        })
        .collect::<Vec<_>>();

    let suggested_copy_command = if let Some(first) = entries.first() {
        format!(
            "rsync -av {} {} {} {} {} <destination_dir>/",
            shell_quote(&first.log_path),
            shell_quote(&first.out_path),
            shell_quote(&first.err_path),
            shell_quote(&first.results_path),
            shell_quote(&first.code_path)
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
        let mut perms =
            std::fs::metadata(path).with_context(|| format!("stat {}", path.display()))?.permissions();
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

pub fn decrypt_bundle_to_local(args: &SlurmBundleDecryptArgs) -> Result<SlurmDecryptReport> {
    ensure_private_directory(&args.out_dir)?;
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

pub fn verify_bundle_integrity(args: &SlurmBundleIntegrityCheck) -> Result<SlurmBundleIntegrityReport> {
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

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{
        decrypt_bundle_to_local, submit_campaign, submit_cross_benchmark, submit_domain_benchmark,
        submit_stage_benchmark, verify_bundle_integrity, write_copy_back_manifest,
    };
    use crate::commands::cli::{
        SlurmBundleDecryptArgs, SlurmBundleIntegrityCheck,
        SlurmCopyBackManifestArgs, SlurmSubmitCampaignArgs, SlurmSubmitCrossArgs,
        SlurmSubmitDomainArgs, SlurmSubmitStageArgs,
    };

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
            std::fs::create_dir_all(root.join(name)).expect("create dir");
        }
        let env_path = root.join("campaign.env");
        std::fs::write(&env_path, "BIJUX_SLURM_ACCOUNT=a\nBIJUX_SLURM_PROJECT=p\n")
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
        std::fs::write(&config_path, config).expect("write config");
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
            user_overrides: None,
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
    fn submit_domain_benchmark_filters_domain() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_domain_benchmark(&SlurmSubmitDomainArgs {
            config,
            env_file: None,
            user_overrides: None,
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
            user_overrides: None,
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
            user_overrides: None,
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
    fn submit_campaign_scripts_include_strict_mode_and_dependency_directives() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config,
            env_file: None,
            user_overrides: None,
            mock_submit: true,
            json: false,
        })
        .expect("submit campaign");

        let first_script =
            std::fs::read_to_string(&report.jobs[0].script_path).expect("read script 1");
        assert!(first_script.contains("set -euo pipefail"));
        assert!(!first_script.contains("--dependency=afterok"));

        let second_script =
            std::fs::read_to_string(&report.jobs[1].script_path).expect("read script 2");
        assert!(second_script.contains("set -euo pipefail"));
        assert!(second_script.contains("#SBATCH --dependency=afterok:mock-0001"));
    }

    #[test]
    fn copy_back_manifest_writes_expected_entries() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let out = root.path().join("copy-back.json");
        let report = write_copy_back_manifest(&SlurmCopyBackManifestArgs {
            config,
            env_file: None,
            user_overrides: None,
            out: Some(out.clone()),
            json: false,
        })
        .expect("write manifest");
        assert_eq!(report.entries.len(), 3);
        assert!(report.suggested_copy_command.starts_with("rsync -av "));
        assert!(report.entries.iter().all(|entry| !entry.script_path.is_empty()));
        assert!(out.is_file());
    }

    #[test]
    fn submit_campaign_writes_encrypted_results_and_code_with_sidecars() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config,
            env_file: None,
            user_overrides: None,
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
            user_overrides: None,
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
            user_overrides: None,
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
            user_overrides: None,
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
            json: false,
        })
        .expect("decrypt bundle");
        assert!(std::path::Path::new(&decrypt_report.output_path).is_file());
        let plaintext =
            std::fs::read_to_string(std::path::Path::new(&decrypt_report.output_path)).expect("read");
        assert!(plaintext.contains("\"schema_version\": \"bijux.hpc.results_bundle.v1\""));
    }

    #[test]
    fn verify_bundle_integrity_reports_ok_for_valid_bundle() {
        let root = tempfile::tempdir().expect("tempdir");
        let config = write_campaign(root.path());
        let report = submit_campaign(&SlurmSubmitCampaignArgs {
            config,
            env_file: None,
            user_overrides: None,
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
            user_overrides: None,
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
}
