use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path};

use anyhow::{bail, Context, Result};
use chrono::DateTime;

use super::{
    AdmittedGrypeDatabase, GrypeDatabaseCompletion, GrypeDatabaseContract, GrypeDatabaseStatus,
    GRYPE_DATABASE_COMPLETION_SCHEMA_V1, GRYPE_DATABASE_CONTRACT_SCHEMA_V1,
};

/// Load and validate the declarative Grype database acquisition authority.
///
/// # Errors
/// Returns an error when the authority is malformed, mutable, or incomplete.
pub fn load_grype_database_contract(path: &Path) -> Result<GrypeDatabaseContract> {
    let contract: GrypeDatabaseContract = toml::from_str(
        &fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))?;
    if contract.schema_version != GRYPE_DATABASE_CONTRACT_SCHEMA_V1
        || contract.grype_version.trim().is_empty()
        || contract.update_url != "https://grype.anchore.io/databases"
        || contract.schema_major == 0
        || contract.maximum_age_days == 0
        || contract.maximum_age_days > 30
    {
        bail!("Grype database contract identity is invalid");
    }
    validate_sha256(&contract.acquisition_script_sha256, "acquisition script")?;
    validate_sha256(&contract.grype_sha256, "Grype binary")?;
    Ok(contract)
}

/// Admit a frozen Grype database acquisition and return its policy-facing identity.
///
/// # Errors
/// Returns an error for source, binary, freshness, status, file-set, or digest drift.
pub fn verify_grype_database(
    contract_path: &Path,
    scanner_root: &Path,
    evidence_root: &Path,
) -> Result<AdmittedGrypeDatabase> {
    let contract = load_grype_database_contract(contract_path)?;
    verify_digest(
        &evidence_root.join("job.sbatch"),
        &contract.acquisition_script_sha256,
        "acquisition script",
    )?;
    verify_digest(&scanner_root.join("bin/grype"), &contract.grype_sha256, "Grype binary")?;

    let completion_path = evidence_root.join("completion.json");
    let completion: GrypeDatabaseCompletion = serde_json::from_slice(
        &fs::read(&completion_path)
            .with_context(|| format!("read {}", completion_path.display()))?,
    )?;
    let completed_at = DateTime::parse_from_rfc3339(&completion.completed_at)
        .context("parse Grype database completion timestamp")?;
    if completion.schema_version != GRYPE_DATABASE_COMPLETION_SCHEMA_V1
        || completion.attempt_id.trim().is_empty()
        || completion.slurm_job_id.is_empty()
        || !completion.slurm_job_id.bytes().all(|byte| byte.is_ascii_digit())
        || completion.grype_version != contract.grype_version
        || completion.grype_sha256 != contract.grype_sha256
    {
        bail!("Grype database completion identity is invalid");
    }
    validate_sha256(&completion.status_sha256, "status")?;
    validate_sha256(&completion.database_manifest_sha256, "database manifest")?;
    verify_digest(&evidence_root.join("status.json"), &completion.status_sha256, "status")?;
    verify_digest(
        &evidence_root.join("database-manifest.sha256"),
        &completion.database_manifest_sha256,
        "database manifest",
    )?;

    let status: GrypeDatabaseStatus =
        serde_json::from_slice(&fs::read(evidence_root.join("status.json"))?)?;
    let expected_schema = format!("v{}.", contract.schema_major);
    let expected_source = format!("{}/v{}/", contract.update_url, contract.schema_major);
    let source_checksum = status
        .from
        .strip_prefix(&expected_source)
        .and_then(|value| value.split_once("?checksum=sha256%3A"))
        .map(|(_, checksum)| checksum)
        .context("Grype database source URL lacks its archive checksum")?;
    validate_sha256(source_checksum, "source archive")?;
    let built = DateTime::parse_from_rfc3339(&status.built)
        .context("parse Grype database build timestamp")?;
    let age = completed_at.signed_duration_since(built);
    if !status.valid
        || !status.schema_version.starts_with(&expected_schema)
        || status.path
            != evidence_root
                .join(format!("database/{}/vulnerability.db", contract.schema_major))
                .to_string_lossy()
        || age.num_seconds() < 0
        || age.num_seconds() > i64::from(contract.maximum_age_days) * 86_400
    {
        bail!("Grype database status is invalid, stale, or path-inconsistent");
    }

    let database_manifest = load_manifest(&evidence_root.join("database-manifest.sha256"))?;
    let required_database = BTreeSet::from([
        format!("database/{}/import.json", contract.schema_major),
        format!("database/{}/last_update_check", contract.schema_major),
        format!("database/{}/vulnerability.db", contract.schema_major),
    ]);
    if database_manifest.keys().cloned().collect::<BTreeSet<_>>() != required_database {
        bail!("Grype database manifest does not contain the exact schema file set");
    }
    for (path, digest) in &database_manifest {
        verify_digest(&evidence_root.join(path), digest, "database file")?;
    }
    let database_path = format!("database/{}/vulnerability.db", contract.schema_major);
    let database_sha256 = database_manifest
        .get(&database_path)
        .context("Grype database manifest omits vulnerability.db")?
        .clone();

    let manifest = load_manifest(&evidence_root.join("manifest.sha256"))?;
    let mut required = required_database;
    required.extend([
        "completion.json".to_string(),
        "database-manifest.sha256".to_string(),
        "job.sbatch".to_string(),
        "providers.txt".to_string(),
        "status.json".to_string(),
    ]);
    if manifest.keys().cloned().collect::<BTreeSet<_>>() != required {
        bail!("Grype acquisition manifest contains an unexpected path");
    }
    for (path, digest) in &manifest {
        verify_digest(&evidence_root.join(path), digest, "acquisition manifest entry")?;
    }
    if fs::read_to_string(evidence_root.join("providers.txt"))?.trim().is_empty() {
        bail!("Grype database provider inventory is empty");
    }
    Ok(AdmittedGrypeDatabase { completion, status, database_sha256 })
}

fn load_manifest(path: &Path) -> Result<BTreeMap<String, String>> {
    let mut entries = BTreeMap::new();
    for line in fs::read_to_string(path)?.lines() {
        let Some((digest, relative)) = line.split_once("  ") else {
            bail!("Grype database manifest has a malformed row");
        };
        if !safe_relative_path(relative)
            || entries.insert(relative.to_string(), digest.to_string()).is_some()
        {
            bail!("Grype database manifest has an unsafe or duplicate path");
        }
        validate_sha256(digest, "manifest digest")?;
    }
    Ok(entries)
}

fn verify_digest(path: &Path, expected: &str, label: &str) -> Result<()> {
    let observed = bijux_dna_infra::hash_file_sha256(path)
        .with_context(|| format!("hash {label} {}", path.display()))?;
    if observed != expected {
        bail!("Grype database {label} digest differs at {}", path.display());
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("Grype database {label} is not a SHA-256 digest");
    }
    Ok(())
}

fn safe_relative_path(value: &str) -> bool {
    !value.is_empty()
        && Path::new(value).components().all(|part| matches!(part, Component::Normal(_)))
}
