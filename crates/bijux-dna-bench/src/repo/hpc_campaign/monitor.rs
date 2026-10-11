//! Owner: benchmark campaign monitoring repository observations and submission projections.
use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_model::sidecar_path_for;
use bijux_dna_bench_model::{
    CampaignDryRunReport, SlurmMonitorEntry, CAMPAIGN_MONITOR_SCHEMA_VERSION,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

// This preserves the historical preview marker. It is neither decryption nor integrity admission.
fn is_encrypted_bundle_marker(path: &Path) -> bool {
    let Ok(file) = std::fs::File::open(path) else {
        return false;
    };
    let mut bytes = Vec::with_capacity(512);
    if file.take(512).read_to_end(&mut bytes).is_err() {
        return false;
    }
    String::from_utf8_lossy(&bytes).contains("\"schema_version\": \"bijux.hpc.bundle.")
}
fn physical_path(workspace_root: &Path, declared: &str) -> PathBuf {
    let path = Path::new(declared);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
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
    #[serde(default)]
    schema_version: Option<String>,
    jobs: Vec<SubmissionManifestJobRow>,
}

fn submission_jobs_from_manifest(
    path: &Path,
) -> Result<BTreeMap<String, SubmissionManifestJobRow>> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let manifest: SubmissionManifestFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if manifest
        .schema_version
        .as_deref()
        .is_some_and(|version| version != CAMPAIGN_MONITOR_SCHEMA_VERSION)
    {
        return Err(anyhow!("unsupported campaign submission observation schema"));
    }
    let mut jobs = BTreeMap::new();
    for row in manifest.jobs {
        let planned_id = row
            .planned_job_id
            .as_deref()
            .filter(|id| !id.trim().is_empty())
            .ok_or_else(|| anyhow!("submission observation is missing planned job identity"))?
            .to_owned();
        if jobs.insert(planned_id.clone(), row).is_some() {
            return Err(anyhow!("duplicate submission observation for planned job {planned_id}"));
        }
    }
    Ok(jobs)
}

pub(crate) fn observe_campaign_jobs(
    workspace_root: &Path,
    report: &CampaignDryRunReport,
    submission_manifest: Option<&Path>,
) -> Result<Vec<SlurmMonitorEntry>> {
    let submitted_jobs = if let Some(path) = submission_manifest {
        let path = if path.is_absolute() { path.to_path_buf() } else { workspace_root.join(path) };
        submission_jobs_from_manifest(&path)?
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
        let log_exists = physical_path(workspace_root, log_path).is_file();
        let out_exists = physical_path(workspace_root, out_path).is_file();
        let err_exists = physical_path(workspace_root, err_path).is_file();
        let results_exists = physical_path(workspace_root, results_path).is_file();
        let code_exists = physical_path(workspace_root, code_path).is_file();
        let results_sidecar_exists =
            sidecar_path_for(&physical_path(workspace_root, results_path)).is_file();
        let code_sidecar_exists =
            sidecar_path_for(&physical_path(workspace_root, code_path)).is_file();
        let results_bundle_encrypted = results_exists
            && is_encrypted_bundle_marker(&physical_path(workspace_root, results_path));
        let code_bundle_encrypted =
            code_exists && is_encrypted_bundle_marker(&physical_path(workspace_root, code_path));
        let appraiser_done =
            physical_path(workspace_root, &format!("{results_path}.appraiser.done")).is_file();
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
    Ok(entries)
}

#[cfg(test)]
#[path = "../../../tests/contracts/campaign/monitor.rs"]
mod tests;
