//! Owner: bijux-dna-analyze
//! Compose and publish evidence, methods, and profile bundles.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::Utc;

use super::construction::build_evidence_bundle;
use super::verification::{
    required_file_paths, validate_evidence_bundle_profile, verify_evidence_bundle,
};

use super::model::{
    EvidenceArchiveMigrationV1, EvidenceBundleFileDigestV1, EvidenceBundleProfileV1,
    EvidenceBundleV1, EvidenceProfileBundleV1,
};

/// Write a deterministic evidence bundle JSON to the run root.
///
/// # Errors
/// Returns an error if bundle construction or writing fails.
pub fn write_evidence_bundle_json(base_dir: &Path, facts_path: Option<&Path>) -> Result<PathBuf> {
    let bundle = build_evidence_bundle(base_dir, facts_path)?;
    let path = base_dir.join("evidence_bundle.json");
    bijux_dna_infra::atomic_write_json(&path, &bundle)
        .with_context(|| format!("write evidence bundle {}", path.display()))?;
    Ok(path)
}

/// Write a deterministic methods-summary JSON generated from evidence inputs.
///
/// # Errors
/// Returns an error if evidence construction or writing fails.
pub fn write_methods_summary_json(base_dir: &Path, facts_path: Option<&Path>) -> Result<PathBuf> {
    let bundle = build_evidence_bundle(base_dir, facts_path)?;
    let Some(summary) = bundle.methods_summary else {
        return Err(anyhow!("methods summary generation failed"));
    };
    let path = base_dir.join("methods_summary.json");
    bijux_dna_infra::atomic_write_json(&path, &summary)
        .with_context(|| format!("write methods summary {}", path.display()))?;
    Ok(path)
}

/// Write a profile-specific bundle suitable for publication, collaboration, or retention archives.
///
/// # Errors
/// Returns an error if evidence construction, verification, or writing fails.
pub fn write_profile_bundle_json(
    base_dir: &Path,
    facts_path: Option<&Path>,
    profile: EvidenceBundleProfileV1,
) -> Result<PathBuf> {
    let bundle = build_profile_bundle(base_dir, facts_path, profile)?;
    let path = base_dir.join(profile_bundle_file_name(profile));
    bijux_dna_infra::atomic_write_json(&path, &bundle)
        .with_context(|| format!("write profile bundle {}", path.display()))?;
    Ok(path)
}

fn build_profile_bundle(
    base_dir: &Path,
    facts_path: Option<&Path>,
    profile: EvidenceBundleProfileV1,
) -> Result<EvidenceProfileBundleV1> {
    let evidence_bundle_path = write_evidence_bundle_json(base_dir, facts_path)?;
    let raw = std::fs::read_to_string(&evidence_bundle_path)
        .with_context(|| format!("read {}", evidence_bundle_path.display()))?;
    let source_bundle: EvidenceBundleV1 = serde_json::from_str(&raw)
        .with_context(|| format!("parse {}", evidence_bundle_path.display()))?;
    let evidence_verification = verify_evidence_bundle(&evidence_bundle_path)?;
    let profile_validation = validate_evidence_bundle_profile(&source_bundle, profile);
    let required_files = build_required_files(base_dir, &source_bundle)?;
    let archive_migration = matches!(profile, EvidenceBundleProfileV1::ArchiveRetention)
        .then(|| build_archive_migration(base_dir, &source_bundle))
        .transpose()?;
    let signature_path = base_dir
        .join("bundle_signature.json")
        .exists()
        .then_some("bundle_signature.json".to_string());
    let evidence_bundle = if matches!(profile, EvidenceBundleProfileV1::CollaboratorRedacted) {
        redact_for_collaborator(source_bundle.clone())
    } else {
        source_bundle.clone()
    };
    Ok(EvidenceProfileBundleV1 {
        schema_version: "bijux.profile_bundle.v1".to_string(),
        profile,
        generated_at: Utc::now().to_rfc3339(),
        run_id: evidence_bundle.run_id.clone(),
        correlation_id: evidence_bundle.correlation_id.clone(),
        evidence_bundle,
        evidence_verification,
        profile_validation,
        required_files,
        signature_path,
        archive_migration,
    })
}

fn build_required_files(
    base_dir: &Path,
    bundle: &EvidenceBundleV1,
) -> Result<Vec<EvidenceBundleFileDigestV1>> {
    let mut rows = Vec::new();
    for rel in required_file_paths(base_dir, bundle) {
        let full = base_dir.join(&rel);
        if !full.exists() {
            continue;
        }
        rows.push(EvidenceBundleFileDigestV1 {
            path: rel,
            sha256: bijux_dna_infra::hash_file_sha256(&full)
                .with_context(|| format!("hash {}", full.display()))?,
        });
    }
    rows.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(rows)
}

fn redact_for_collaborator(bundle: EvidenceBundleV1) -> EvidenceBundleV1 {
    fn redact_path(path: Option<&str>) -> Option<String> {
        path.map(std::path::Path::new)
            .and_then(|path| path.file_name().and_then(|name| name.to_str()))
            .map(str::to_string)
    }
    let mut redacted = bundle;
    redacted.sources.manifest_path = redact_path(redacted.sources.manifest_path.as_deref());
    redacted.sources.plan_manifest_path =
        redact_path(redacted.sources.plan_manifest_path.as_deref());
    redacted.sources.report_path = redact_path(redacted.sources.report_path.as_deref());
    redacted.sources.run_summary_path = redact_path(redacted.sources.run_summary_path.as_deref());
    redacted.sources.facts_path = redact_path(redacted.sources.facts_path.as_deref());
    redacted.sources.graph_path = redact_path(redacted.sources.graph_path.as_deref());
    redacted.sources.environment_path = redact_path(redacted.sources.environment_path.as_deref());
    redacted.sources.runtime_policy_path =
        redact_path(redacted.sources.runtime_policy_path.as_deref());
    redacted.sources.run_state_path = redact_path(redacted.sources.run_state_path.as_deref());
    redacted.sources.executor_descriptor_path =
        redact_path(redacted.sources.executor_descriptor_path.as_deref());
    redacted.sources.checkpoint_path = redact_path(redacted.sources.checkpoint_path.as_deref());
    redacted.sources.failure_path = redact_path(redacted.sources.failure_path.as_deref());
    redacted.sources.artifact_inventory_path =
        redact_path(redacted.sources.artifact_inventory_path.as_deref());
    redacted.sources.replay_manifest_path =
        redact_path(redacted.sources.replay_manifest_path.as_deref());
    redacted.sources.hash_ledger_path = redact_path(redacted.sources.hash_ledger_path.as_deref());
    redacted.sources.evidence_verification_path =
        redact_path(redacted.sources.evidence_verification_path.as_deref());
    redacted.sources.telemetry_paths = redacted
        .sources
        .telemetry_paths
        .iter()
        .map(|value| {
            std::path::Path::new(value)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("redacted")
                .to_string()
        })
        .collect();
    for artifact in &mut redacted.artifacts {
        artifact.path = std::path::Path::new(&artifact.path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("redacted")
            .to_string();
    }
    redacted
}

fn build_archive_migration(
    base_dir: &Path,
    bundle: &EvidenceBundleV1,
) -> Result<EvidenceArchiveMigrationV1> {
    let manifest_schema_version = load_schema_version(
        base_dir,
        bundle.sources.manifest_path.as_deref(),
        "bijux.run_manifest.v3",
    )?;
    let evidence_schema_version = bundle.schema_version.clone();
    let artifact_inventory_schema_version = load_schema_version(
        base_dir,
        bundle.sources.artifact_inventory_path.as_deref(),
        "bijux.artifact_inventory.v1",
    )?;
    let hash_ledger_schema_version = load_schema_version(
        base_dir,
        bundle.sources.hash_ledger_path.as_deref(),
        "bijux.hash_ledger.v1",
    )?;
    Ok(EvidenceArchiveMigrationV1 {
        manifest_schema_version,
        evidence_schema_version,
        artifact_inventory_schema_version: Some(artifact_inventory_schema_version),
        hash_ledger_schema_version: Some(hash_ledger_schema_version),
    })
}

fn load_schema_version(
    base_dir: &Path,
    relative_path: Option<&str>,
    fallback: &str,
) -> Result<String> {
    let Some(relative_path) = relative_path else {
        return Ok(fallback.to_string());
    };
    let full = base_dir.join(relative_path);
    let raw = std::fs::read_to_string(&full).with_context(|| format!("read {}", full.display()))?;
    let value: serde_json::Value =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", full.display()))?;
    Ok(value
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(fallback)
        .to_string())
}

fn profile_bundle_file_name(profile: EvidenceBundleProfileV1) -> &'static str {
    match profile {
        EvidenceBundleProfileV1::Draft => "profile_bundle_draft.json",
        EvidenceBundleProfileV1::Operational => "profile_bundle_operational.json",
        EvidenceBundleProfileV1::Certification => "profile_bundle_certification.json",
        EvidenceBundleProfileV1::Publication => "profile_bundle_publication.json",
        EvidenceBundleProfileV1::PublicationStrict => "profile_bundle_publication_strict.json",
        EvidenceBundleProfileV1::CollaboratorRedacted => {
            "profile_bundle_collaborator_redacted.json"
        }
        EvidenceBundleProfileV1::ArchiveRetention => "profile_bundle_archive_retention.json",
    }
}
