//! Admission rules for selected container run evidence.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path};

use anyhow::{bail, Context, Result};

use super::{
    RunAttemptLedger, RunEvidenceIndex, RunEvidenceLock, CONTAINER_RUN_ATTEMPT_LEDGER_SCHEMA_V1,
    CONTAINER_RUN_EVIDENCE_INDEX_SCHEMA_V2, CONTAINER_RUN_EVIDENCE_LOCK_SCHEMA_V1,
};

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

/// Validate every selected run lock and its copied evidence files.
///
/// Campaign directories are ignored when they contain `config-campaign.json`.
///
/// # Errors
/// Returns an error when a run directory, lock identity, inventory, or copied file is invalid.
pub fn validate_run_evidence_root(root: &Path) -> Result<Vec<RunEvidenceLock>> {
    let mut records = Vec::new();
    let mut identities = BTreeSet::new();
    for path in child_directories(root)? {
        if !path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("run-"))
        {
            continue;
        }
        if path.join("config-campaign.json").is_file() {
            continue;
        }
        if !path.join("evidence.lock.json").is_file() {
            bail!("run directory lacks governed evidence: {}", path.display());
        }
        let record = load_run_evidence_lock(&path)?;
        if !identities.insert(record.run_id.clone()) {
            bail!("duplicate tracked run identity: {}", record.run_id);
        }
        records.push(record);
    }
    records.sort_by_key(|record| run_number(&record.run_id).unwrap_or(u64::MAX));
    Ok(records)
}

/// Validate a superseded-attempt ledger against the selected run records.
///
/// # Errors
/// Returns an error when identities, replacements, hashes, or summary counts drift.
pub fn validate_run_attempt_ledger(
    ledger_path: &Path,
    selected_records: &[RunEvidenceLock],
) -> Result<RunAttemptLedger> {
    let raw = fs::read_to_string(ledger_path)
        .with_context(|| format!("read run-attempt ledger {}", ledger_path.display()))?;
    let ledger: RunAttemptLedger = serde_json::from_str(&raw)
        .with_context(|| format!("parse run-attempt ledger {}", ledger_path.display()))?;
    validate_run_attempt_ledger_value(
        &ledger,
        selected_records,
        &ledger_path.display().to_string(),
    )?;
    Ok(ledger)
}

pub(super) fn validate_run_attempt_ledger_value(
    ledger: &RunAttemptLedger,
    selected_records: &[RunEvidenceLock],
    label: &str,
) -> Result<()> {
    if ledger.schema_version != CONTAINER_RUN_ATTEMPT_LEDGER_SCHEMA_V1 {
        bail!("unsupported attempt ledger schema: {label}");
    }
    let final_by_id = selected_records
        .iter()
        .map(|record| (record.run_id.as_str(), record))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut execution_status = BTreeMap::new();
    for attempt in &ledger.attempts {
        if !valid_run_id(&attempt.run_id) {
            bail!("attempt ledger run identity is invalid: {}", attempt.run_id);
        }
        if !seen.insert(attempt.run_id.as_str())
            || final_by_id.contains_key(attempt.run_id.as_str())
        {
            bail!("attempt ledger run identity is duplicated: {}", attempt.run_id);
        }
        let Some(final_record) = final_by_id.get(attempt.superseded_by.as_str()) else {
            bail!("attempt ledger replacement is invalid: {}", attempt.run_id);
        };
        if final_record.tool != attempt.tool {
            bail!("attempt ledger replacement is invalid: {}", attempt.run_id);
        }
        validate_hex_identity(&attempt.evidence_lock_sha256, 64, "evidence lock", &attempt.run_id)?;
        validate_hex_identity(
            &attempt.repository_commit,
            40,
            "repository commit",
            &attempt.run_id,
        )?;
        validate_source_result(&attempt.source_result, &attempt.run_id)?;
        if attempt.source_file_count == 0 {
            bail!("attempt {} has no source evidence files", attempt.run_id);
        }
        *execution_status.entry(attempt.execution_status.as_str().to_string()).or_insert(0) += 1;
    }
    if ledger.summary.published_run_count != selected_records.len() {
        bail!("attempt ledger published count drifted: {label}");
    }
    if ledger.summary.superseded_attempt_count != ledger.attempts.len() {
        bail!("attempt ledger attempt count drifted: {label}");
    }
    if ledger.summary.execution_status != execution_status {
        bail!("attempt ledger execution status drifted: {label}");
    }
    Ok(())
}

/// Load and validate one selected run-evidence lock and every tracked file it binds.
///
/// # Errors
/// Returns an error when the directory identity, lock schema, source identities, or tracked bytes
/// are invalid.
pub fn load_run_evidence_lock(path: &Path) -> Result<RunEvidenceLock> {
    let (run_id, tool) = run_directory_identity(path)?;
    let lock_path = path.join("evidence.lock.json");
    let raw = fs::read_to_string(&lock_path)
        .with_context(|| format!("read run evidence lock {}", lock_path.display()))?;
    let record: RunEvidenceLock = serde_json::from_str(&raw)
        .with_context(|| format!("parse run evidence lock {}", lock_path.display()))?;
    if record.schema_version != CONTAINER_RUN_EVIDENCE_LOCK_SCHEMA_V1 {
        bail!("unsupported evidence lock schema: {}", path.display());
    }
    if record.run_id != run_id || record.tool != tool {
        bail!("tracked directory and evidence lock disagree: {}", path.display());
    }
    validate_hex_identity(&record.repository_commit, 40, "repository commit", &record.run_id)?;
    validate_source_result(&record.source_result, &record.run_id)?;
    if record.source_files.is_empty() {
        bail!("source file inventory is absent: {}", path.display());
    }
    let mut source_paths = BTreeSet::new();
    for source in &record.source_files {
        let source_path = source
            .path
            .as_deref()
            .with_context(|| format!("source file inventory path is absent: {}", path.display()))?;
        validate_relative_path(source_path, "source file inventory", path)?;
        if !source_paths.insert(source_path) {
            bail!("source file inventory path is duplicated: {}", path.display());
        }
        validate_hex_identity(&source.sha256, 64, "source file digest", &record.run_id)?;
    }
    for (name, expected) in &record.tracked_files {
        validate_relative_path(name, "tracked evidence", path)?;
        if expected.path.is_some() {
            bail!("tracked evidence identity embeds an unexpected path: {name}");
        }
        validate_hex_identity(&expected.sha256, 64, "tracked evidence digest", &record.run_id)?;
        let candidate = path.join(name);
        let metadata = candidate
            .metadata()
            .with_context(|| format!("tracked evidence is absent: {}", candidate.display()))?;
        if metadata.len() != expected.size_bytes {
            bail!("tracked evidence size changed: {}", candidate.display());
        }
        if bijux_dna_infra::hash_file_sha256(&candidate)? != expected.sha256 {
            bail!("tracked evidence digest changed: {}", candidate.display());
        }
    }
    let credential = path.join("config-credential.yml");
    if credential.is_file() {
        crate::execution_contract::load_container_credential_reference(&credential)?;
    }
    Ok(record)
}

pub(super) fn run_directory_identity(path: &Path) -> Result<(String, String)> {
    let name = path.file_name().and_then(|value| value.to_str()).with_context(|| {
        format!("tracked run directory has invalid identity: {}", path.display())
    })?;
    let Some(rest) = name.strip_prefix("run-") else {
        bail!("tracked run directory has invalid identity: {}", path.display());
    };
    let Some((number, tool)) = rest.split_once('-') else {
        bail!("tracked run directory has invalid identity: {}", path.display());
    };
    let run_id = format!("run-{number}");
    if !valid_run_id(&run_id)
        || tool.is_empty()
        || !tool
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        bail!("tracked run directory has invalid identity: {}", path.display());
    }
    Ok((run_id, tool.to_string()))
}

pub(super) fn run_number(run_id: &str) -> Result<u64> {
    if !valid_run_id(run_id) {
        bail!("invalid run identity: {run_id}");
    }
    run_id[4..].parse::<u64>().with_context(|| format!("parse run identity {run_id}"))
}

fn child_directories(root: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut paths = fs::read_dir(root)
        .with_context(|| format!("read run evidence root {}", root.display()))?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.retain(|path| path.is_dir());
    paths.sort();
    Ok(paths)
}

fn validate_relative_path(value: &str, label: &str, root: &Path) -> Result<()> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
        })
    {
        bail!("{label} path is invalid: {}", root.display());
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
