//! Admission rules for selected container run evidence.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path};

use anyhow::{bail, Context, Result};

use super::{RunEvidenceIndex, CONTAINER_RUN_EVIDENCE_INDEX_SCHEMA_V2};

/// Load and validate one portable selected run-evidence index.
///
/// # Errors
/// Returns an error when the index cannot be read, parsed, or admitted.
pub fn load_run_evidence_index(path: &Path) -> Result<RunEvidenceIndex> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read run-evidence index {}", path.display()))?;
    let index: RunEvidenceIndex = serde_json::from_str(&raw)
        .with_context(|| format!("parse run-evidence index {}", path.display()))?;
    validate_run_evidence_index(&index)
        .with_context(|| format!("validate run-evidence index {}", path.display()))?;
    Ok(index)
}

/// Validate selected run ownership, immutable provenance, and summary integrity.
///
/// # Errors
/// Returns an error when a run is duplicated, provenance is incomplete, or the summary drifted.
pub fn validate_run_evidence_index(index: &RunEvidenceIndex) -> Result<()> {
    if index.schema_version != CONTAINER_RUN_EVIDENCE_INDEX_SCHEMA_V2 {
        bail!("unsupported run-evidence index schema: {}", index.schema_version);
    }
    let mut run_ids = BTreeSet::new();
    let mut tools = BTreeSet::new();
    let mut execution_status = BTreeMap::new();
    let mut scientific_status = BTreeMap::new();
    for record in &index.runs {
        if !valid_run_id(&record.run_id) {
            bail!("run-evidence index has invalid run ID {}", record.run_id);
        }
        if !run_ids.insert(record.run_id.as_str()) {
            bail!("run-evidence index duplicates run {}", record.run_id);
        }
        for (value, label) in [
            (&record.campaign, "campaign"),
            (&record.corpus, "corpus"),
            (&record.domain, "domain"),
            (&record.pipeline_schema_version, "pipeline schema"),
            (&record.sample, "sample"),
            (&record.slurm_job_id, "Slurm job ID"),
            (&record.stage, "stage"),
            (&record.tool, "tool"),
        ] {
            if value.trim().is_empty() {
                bail!("run {} has an empty {label}", record.run_id);
            }
        }
        validate_hex_identity(&record.evidence_lock_sha256, 64, "evidence lock", &record.run_id)?;
        validate_hex_identity(&record.repository_commit, 40, "repository commit", &record.run_id)?;
        validate_source_result(&record.source_result, &record.run_id)?;
        if record.source_file_count == 0 {
            bail!("run {} has no source evidence files", record.run_id);
        }
        tools.insert(record.tool.as_str());
        *execution_status.entry(record.execution_status.as_str().to_string()).or_insert(0) += 1;
        *scientific_status.entry(record.scientific_status.as_str().to_string()).or_insert(0) += 1;
    }
    if index.summary.run_count != index.runs.len() {
        bail!("run-evidence summary run count drifted");
    }
    if index.summary.tool_count != tools.len() {
        bail!("run-evidence summary tool count drifted");
    }
    if index.summary.execution_status != execution_status {
        bail!("run-evidence summary execution status drifted");
    }
    if index.summary.scientific_status != scientific_status {
        bail!("run-evidence summary scientific status drifted");
    }
    Ok(())
}

fn valid_run_id(value: &str) -> bool {
    value.strip_prefix("run-").is_some_and(|number| {
        number.len() >= 4
            && !number.starts_with('0')
            && number.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn validate_source_result(value: &str, run_id: &str) -> Result<()> {
    let path = Path::new(value);
    if path.is_absolute()
        || !value.starts_with("results/container-validation/")
        || path.components().any(|component| {
            matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
        })
    {
        bail!("run {run_id} has a non-portable source result: {value}");
    }
    Ok(())
}

fn validate_hex_identity(value: &str, length: usize, label: &str, run_id: &str) -> Result<()> {
    if value.len() != length
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        || value.bytes().all(|byte| byte == b'0')
    {
        bail!("run {run_id} has an unresolved {label}: {value}");
    }
    Ok(())
}
