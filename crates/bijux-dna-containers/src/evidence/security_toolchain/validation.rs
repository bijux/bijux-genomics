use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path};

use anyhow::{bail, Context, Result};

use super::{
    SecurityScannerToolchainCompletion, SecurityScannerToolchainContract,
    SECURITY_SCANNER_TOOLCHAIN_COMPLETION_SCHEMA_V1, SECURITY_SCANNER_TOOLCHAIN_CONTRACT_SCHEMA_V1,
};

/// Load and validate the declarative scanner toolchain authority.
///
/// # Errors
/// Returns an error when the contract is malformed, incomplete, or mutable.
pub fn load_security_scanner_toolchain_contract(
    path: &Path,
) -> Result<SecurityScannerToolchainContract> {
    let contract: SecurityScannerToolchainContract = toml::from_str(
        &fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))?;
    if contract.schema_version != SECURITY_SCANNER_TOOLCHAIN_CONTRACT_SCHEMA_V1 {
        bail!("unsupported security scanner toolchain contract schema");
    }
    validate_sha256(&contract.acquisition_script_sha256, "acquisition script")?;
    let mut ids = BTreeSet::new();
    for tool in &contract.tools {
        if !matches!(tool.id.as_str(), "syft" | "grype" | "gitleaks")
            || !ids.insert(tool.id.as_str())
        {
            bail!("security scanner toolchain has an invalid or duplicate tool ID");
        }
        if tool.version.trim().is_empty()
            || !safe_name(&tool.archive)
            || !safe_name(&tool.checksums)
            || !tool.release.starts_with("https://github.com/")
            || tool.release.contains("/latest")
        {
            bail!("security scanner {} has mutable or unsafe release identity", tool.id);
        }
        validate_sha256(&tool.archive_sha256, "scanner archive")?;
        validate_sha256(&tool.checksums_sha256, "scanner checksums")?;
    }
    if ids != BTreeSet::from(["gitleaks", "grype", "syft"]) {
        bail!("security scanner toolchain must contain syft, grype, and gitleaks");
    }
    Ok(contract)
}

/// Verify an immutable scanner acquisition against the governed contract and file manifest.
///
/// # Errors
/// Returns an error for missing, extra, malformed, or digest-mismatched evidence.
pub fn verify_security_scanner_toolchain(
    contract_path: &Path,
    evidence_root: &Path,
) -> Result<SecurityScannerToolchainCompletion> {
    let contract = load_security_scanner_toolchain_contract(contract_path)?;
    verify_digest(
        &evidence_root.join("job.sbatch"),
        &contract.acquisition_script_sha256,
        "acquisition script",
    )?;
    let completion_path = evidence_root.join("completion.json");
    let completion: SecurityScannerToolchainCompletion = serde_json::from_slice(
        &fs::read(&completion_path)
            .with_context(|| format!("read {}", completion_path.display()))?,
    )?;
    if completion.schema_version != SECURITY_SCANNER_TOOLCHAIN_COMPLETION_SCHEMA_V1
        || completion.attempt_id.trim().is_empty()
        || completion.completed_at.trim().is_empty()
        || !completion.slurm_job_id.bytes().all(|byte| byte.is_ascii_digit())
    {
        bail!("security scanner completion identity is invalid");
    }
    let manifest = load_manifest(&evidence_root.join("manifest.sha256"))?;
    let mut required = BTreeSet::from([
        "completion.json".to_string(),
        "syft-version.txt".to_string(),
        "grype-version.txt".to_string(),
        "gitleaks-version.txt".to_string(),
    ]);
    for tool in &contract.tools {
        required.extend([
            format!("bin/{}", tool.id),
            format!("downloads/{}", tool.archive),
            format!("downloads/{}", tool.checksums),
        ]);
        verify_digest(
            &evidence_root.join("downloads").join(&tool.archive),
            &tool.archive_sha256,
            "scanner archive",
        )?;
        verify_digest(
            &evidence_root.join("downloads").join(&tool.checksums),
            &tool.checksums_sha256,
            "scanner checksums",
        )?;
        let binary = completion
            .binaries
            .get(&tool.id)
            .with_context(|| format!("completion omits {}", tool.id))?;
        if binary.version != tool.version {
            bail!("security scanner {} version differs from contract", tool.id);
        }
        validate_sha256(&binary.sha256, "scanner binary")?;
        verify_digest(&evidence_root.join("bin").join(&tool.id), &binary.sha256, "scanner binary")?;
    }
    if completion.binaries.len() != contract.tools.len() {
        bail!("security scanner completion contains an unexpected binary");
    }
    for path in &required {
        let expected = manifest
            .get(path)
            .with_context(|| format!("security scanner manifest omits {path}"))?;
        verify_digest(&evidence_root.join(path), expected, "manifest entry")?;
    }
    if manifest.keys().cloned().collect::<BTreeSet<_>>() != required {
        bail!("security scanner manifest contains an unexpected path");
    }
    Ok(completion)
}

fn load_manifest(path: &Path) -> Result<BTreeMap<String, String>> {
    let mut entries = BTreeMap::new();
    for line in fs::read_to_string(path)?.lines() {
        let Some((digest, relative)) = line.split_once("  ") else {
            bail!("security scanner manifest has a malformed row");
        };
        if !safe_relative_path(relative)
            || entries.insert(relative.to_string(), digest.to_string()).is_some()
        {
            bail!("security scanner manifest has an unsafe or duplicate path");
        }
        validate_sha256(digest, "manifest digest")?;
    }
    Ok(entries)
}

fn verify_digest(path: &Path, expected: &str, label: &str) -> Result<()> {
    let observed = bijux_dna_infra::hash_file_sha256(path)
        .with_context(|| format!("hash {label} {}", path.display()))?;
    if observed != expected {
        bail!("security scanner {label} digest differs at {}", path.display());
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("security scanner {label} is not a SHA-256 digest");
    }
    Ok(())
}

fn safe_name(value: &str) -> bool {
    !value.is_empty() && Path::new(value).components().count() == 1
}

fn safe_relative_path(value: &str) -> bool {
    !value.is_empty()
        && Path::new(value).components().all(|part| matches!(part, Component::Normal(_)))
}
