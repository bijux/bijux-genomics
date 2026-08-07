//! Typed admission for selected container run-evidence publications.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

pub const CONTAINER_RUN_EVIDENCE_INDEX_SCHEMA_V2: &str = "bijux.container.run_evidence_index.v2";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunEvidenceIndex {
    pub schema_version: String,
    pub runs: Vec<RunEvidenceRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunEvidenceRecord {
    pub campaign: String,
    pub corpus: String,
    pub domain: String,
    pub evidence_lock_sha256: String,
    pub execution_status: AttemptExecutionStatus,
    pub pipeline_schema_version: String,
    pub repository_commit: String,
    pub run_id: String,
    pub sample: String,
    pub scientific_status: ScientificAcceptanceStatus,
    pub slurm_job_id: String,
    pub source_file_count: u64,
    pub source_result: String,
    pub stage: String,
    pub tool: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptExecutionStatus {
    Passed,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScientificAcceptanceStatus {
    Passed,
    Failed,
    NotAssessed,
}

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

/// Validate selected run ownership, immutable provenance, and scheduler identity.
///
/// # Errors
/// Returns an error when a run is duplicated or has incomplete provenance.
pub fn validate_run_evidence_index(index: &RunEvidenceIndex) -> Result<()> {
    if index.schema_version != CONTAINER_RUN_EVIDENCE_INDEX_SCHEMA_V2 {
        bail!("unsupported run-evidence index schema: {}", index.schema_version);
    }
    let mut run_ids = BTreeSet::new();
    for record in &index.runs {
        if !run_ids.insert(record.run_id.as_str()) {
            bail!("run-evidence index duplicates run {}", record.run_id);
        }
        for (value, label) in [
            (&record.run_id, "run ID"),
            (&record.campaign, "campaign"),
            (&record.corpus, "corpus"),
            (&record.domain, "domain"),
            (&record.pipeline_schema_version, "pipeline schema"),
            (&record.sample, "sample"),
            (&record.slurm_job_id, "Slurm job ID"),
            (&record.source_result, "source result"),
            (&record.stage, "stage"),
            (&record.tool, "tool"),
        ] {
            if value.trim().is_empty() {
                bail!("run {} has an empty {label}", record.run_id);
            }
        }
        validate_hex_identity(&record.evidence_lock_sha256, 64, "evidence lock", &record.run_id)?;
        validate_hex_identity(&record.repository_commit, 40, "repository commit", &record.run_id)?;
        if record.source_file_count == 0 {
            bail!("run {} has no source evidence files", record.run_id);
        }
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
