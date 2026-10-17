//! Owner: bijux-dna-analyze
//! Validate typed profiles and verify acquired evidence and file membership.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use super::model::{
    EvidenceBundleProfileV1, EvidenceBundleV1, EvidenceCheckV1, EvidenceProfileBundleV1,
    EvidenceProfileBundleVerificationV1, EvidenceProfileCheckV1, EvidenceProfileValidationV1,
    EvidenceVerificationV1,
};

/// Verify a profile bundle in an external checkout.
///
/// # Errors
/// Returns an error if bundle parsing fails.
pub fn verify_profile_bundle(
    profile_bundle_path: &Path,
) -> Result<EvidenceProfileBundleVerificationV1> {
    let raw = std::fs::read_to_string(profile_bundle_path)
        .with_context(|| format!("read {}", profile_bundle_path.display()))?;
    let bundle: EvidenceProfileBundleV1 = serde_json::from_str(&raw)
        .with_context(|| format!("parse {}", profile_bundle_path.display()))?;
    let base_dir =
        profile_bundle_path.parent().ok_or_else(|| anyhow!("profile bundle path has no parent"))?;

    let (current_inputs, aliases_valid) = profile_evidence_inputs(base_dir, &bundle);
    let expected_paths = required_file_paths(base_dir, &current_inputs);
    let declared_paths =
        bundle.required_files.iter().map(|item| item.path.clone()).collect::<BTreeSet<_>>();
    let membership_valid =
        expected_paths == declared_paths && declared_paths.len() == bundle.required_files.len();
    let mut missing_paths = expected_paths.difference(&declared_paths).cloned().collect::<Vec<_>>();
    let mut hash_mismatches = Vec::new();
    for item in &bundle.required_files {
        let full = base_dir.join(&item.path);
        if !full.exists() {
            missing_paths.push(item.path.clone());
            continue;
        }
        let actual = bijux_dna_infra::hash_file_sha256(&full)
            .with_context(|| format!("hash required file {}", full.display()))?;
        if actual != item.sha256 {
            hash_mismatches.push(item.path.clone());
        }
    }

    let current_evidence = verify_evidence_bundle_inputs(&current_inputs, base_dir);
    let current_profile = validate_evidence_bundle_profile(&bundle.evidence_bundle, bundle.profile);
    missing_paths.extend(current_evidence.missing_paths);
    missing_paths.sort();
    missing_paths.dedup();
    let evidence_verified = bundle.evidence_verification.verified && current_evidence.verified;
    let profile_valid = bundle.profile_validation.ok && current_profile.ok && aliases_valid;
    let verified = membership_valid
        && missing_paths.is_empty()
        && hash_mismatches.is_empty()
        && evidence_verified
        && profile_valid;
    Ok(EvidenceProfileBundleVerificationV1 {
        schema_version: "bijux.profile_bundle_verification.v1".to_string(),
        verified,
        missing_paths,
        hash_mismatches,
        evidence_verified,
        profile_valid,
    })
}

/// Verify an evidence bundle and its referenced sources/artifacts.
///
/// # Errors
/// Returns an error if the bundle cannot be read or parsed.
pub fn verify_evidence_bundle(bundle_path: &Path) -> Result<EvidenceVerificationV1> {
    let bundle_raw = std::fs::read_to_string(bundle_path)
        .with_context(|| format!("read evidence bundle {}", bundle_path.display()))?;
    let bundle: EvidenceBundleV1 = serde_json::from_str(&bundle_raw)
        .with_context(|| format!("parse evidence bundle {}", bundle_path.display()))?;
    let base_dir =
        bundle_path.parent().ok_or_else(|| anyhow!("evidence bundle missing parent directory"))?;
    Ok(verify_evidence_bundle_inputs(&bundle, base_dir))
}

#[allow(clippy::too_many_lines)]
fn verify_evidence_bundle_inputs(
    bundle: &EvidenceBundleV1,
    base_dir: &Path,
) -> EvidenceVerificationV1 {
    let mut checks = Vec::new();
    let mut missing_paths = Vec::new();
    for (label, path) in [
        ("manifest_path", bundle.sources.manifest_path.as_deref()),
        ("plan_manifest_path", bundle.sources.plan_manifest_path.as_deref()),
        ("report_path", bundle.sources.report_path.as_deref()),
        ("run_summary_path", bundle.sources.run_summary_path.as_deref()),
        ("facts_path", bundle.sources.facts_path.as_deref()),
        ("graph_path", bundle.sources.graph_path.as_deref()),
        ("environment_path", bundle.sources.environment_path.as_deref()),
        ("runtime_policy_path", bundle.sources.runtime_policy_path.as_deref()),
        ("run_state_path", bundle.sources.run_state_path.as_deref()),
        ("executor_descriptor_path", bundle.sources.executor_descriptor_path.as_deref()),
        ("checkpoint_path", bundle.sources.checkpoint_path.as_deref()),
        ("artifact_inventory_path", bundle.sources.artifact_inventory_path.as_deref()),
        ("replay_manifest_path", bundle.sources.replay_manifest_path.as_deref()),
        ("hash_ledger_path", bundle.sources.hash_ledger_path.as_deref()),
    ] {
        if let Some(path) = path {
            let full = base_dir.join(path);
            let ok = full.exists();
            if !ok {
                missing_paths.push(path.to_string());
            }
            checks.push(EvidenceCheckV1 {
                check_id: label.to_string(),
                ok,
                message: if ok {
                    format!("{label} present")
                } else {
                    format!("{label} missing at {path}")
                },
            });
        }
    }
    for path in &bundle.sources.telemetry_paths {
        let full = base_dir.join(path);
        let ok = full.exists();
        if !ok {
            missing_paths.push(path.clone());
        }
        checks.push(EvidenceCheckV1 {
            check_id: format!("telemetry:{path}"),
            ok,
            message: if ok {
                format!("telemetry source present at {path}")
            } else {
                format!("telemetry source missing at {path}")
            },
        });
    }

    for artifact in &bundle.artifacts {
        let full = base_dir.join(&artifact.path);
        let ok = full.exists();
        if !ok {
            missing_paths.push(artifact.path.clone());
        }
        let hash_ok = match (&artifact.sha256, ok) {
            (Some(expected), true) => {
                bijux_dna_infra::hash_file_sha256(&full).is_ok_and(|actual| actual == *expected)
            }
            (None, true) => true,
            (_, false) => false,
        };
        checks.push(EvidenceCheckV1 {
            check_id: format!("artifact:{}", artifact.name),
            ok: ok && hash_ok,
            message: if ok && hash_ok {
                format!("artifact {} verified", artifact.name)
            } else if ok {
                format!("artifact {} hash mismatch", artifact.name)
            } else {
                format!("artifact {} missing at {}", artifact.name, artifact.path)
            },
        });
    }

    if let Some(path) = bundle.sources.artifact_inventory_path.as_deref() {
        let full = base_dir.join(path);
        let (ok, message) = verify_artifact_inventory_contract(&full);
        checks.push(EvidenceCheckV1 {
            check_id: "artifact_inventory_contract".to_string(),
            ok,
            message,
        });
    }
    if let Some(path) = bundle.sources.hash_ledger_path.as_deref() {
        let full = base_dir.join(path);
        let (ok, message) = verify_hash_ledger_contract(base_dir, &full);
        checks.push(EvidenceCheckV1 { check_id: "hash_ledger_contract".to_string(), ok, message });
    }
    if let Some(path) = bundle.sources.report_path.as_deref() {
        let full = base_dir.join(path);
        let (ok, message) = verify_report_completeness(&full);
        checks.push(EvidenceCheckV1 { check_id: "report_completeness".to_string(), ok, message });
    }
    if let (Some(manifest_path), Some(run_state_path)) =
        (bundle.sources.manifest_path.as_deref(), bundle.sources.run_state_path.as_deref())
    {
        let (ok, message) = verify_advisory_and_enforced_consistency(
            &base_dir.join(manifest_path),
            &base_dir.join(run_state_path),
        );
        checks.push(EvidenceCheckV1 {
            check_id: "mode_state_consistency".to_string(),
            ok,
            message,
        });
    }

    let verified = checks.iter().all(|check| check.ok) && bundle.health.auditable;
    EvidenceVerificationV1 {
        schema_version: "bijux.evidence_verification.v1".to_string(),
        verified,
        checks,
        missing_paths,
        gap_count: bundle.health.gaps.len(),
    }
}

#[must_use]
pub fn validate_evidence_bundle_profile(
    bundle: &EvidenceBundleV1,
    profile: EvidenceBundleProfileV1,
) -> EvidenceProfileValidationV1 {
    let mut checks = Vec::new();
    let mut required_paths_present = true;
    for (check_id, ok, message) in profile_requirements(bundle, profile) {
        if !ok {
            required_paths_present = false;
        }
        checks.push(EvidenceProfileCheckV1 { check_id, ok, message });
    }

    let tolerated_gap_codes = tolerated_gap_codes(profile);
    let (tolerated, blocking): (Vec<_>, Vec<_>) = bundle
        .health
        .gaps
        .iter()
        .map(|gap| gap.code.clone())
        .partition(|code| tolerated_gap_codes.contains(code));
    let ok = required_paths_present && blocking.is_empty();

    EvidenceProfileValidationV1 {
        schema_version: "bijux.evidence_profile_validation.v1".to_string(),
        profile,
        ok,
        required_paths_present,
        tolerated_gap_codes: tolerated,
        blocking_gap_codes: blocking,
        checks,
    }
}

fn profile_evidence_inputs(
    base_dir: &Path,
    bundle: &EvidenceProfileBundleV1,
) -> (EvidenceBundleV1, bool) {
    let mut current = bundle.evidence_bundle.clone();
    if !matches!(bundle.profile, EvidenceBundleProfileV1::CollaboratorRedacted) {
        return (current, true);
    }
    let paths =
        bundle.required_files.iter().map(|item| item.path.as_str()).collect::<BTreeSet<_>>();
    let mut aliases_valid = true;
    let mut resolve = |raw: &str| {
        let candidates = paths
            .iter()
            .filter(|path| Path::new(path).file_name() == Path::new(raw).file_name())
            .collect::<Vec<_>>();
        if candidates.len() == 1 {
            (*candidates[0]).to_string()
        } else {
            aliases_valid &= candidates.is_empty() && base_dir.join(raw).is_file();
            raw.to_string()
        }
    };
    // Redacted presentation names require an unambiguous retained physical binding.
    for path in [
        &mut current.sources.manifest_path,
        &mut current.sources.plan_manifest_path,
        &mut current.sources.report_path,
        &mut current.sources.run_summary_path,
        &mut current.sources.facts_path,
        &mut current.sources.graph_path,
        &mut current.sources.environment_path,
        &mut current.sources.runtime_policy_path,
        &mut current.sources.run_state_path,
        &mut current.sources.executor_descriptor_path,
        &mut current.sources.checkpoint_path,
        &mut current.sources.failure_path,
        &mut current.sources.artifact_inventory_path,
        &mut current.sources.replay_manifest_path,
        &mut current.sources.hash_ledger_path,
        &mut current.sources.evidence_verification_path,
    ]
    .into_iter()
    .flatten()
    {
        *path = resolve(path);
    }
    for path in &mut current.sources.telemetry_paths {
        *path = resolve(path);
    }
    for artifact in &mut current.artifacts {
        artifact.path = resolve(&artifact.path);
    }
    (current, aliases_valid)
}

pub(super) fn required_file_paths(base_dir: &Path, bundle: &EvidenceBundleV1) -> BTreeSet<String> {
    let mut required = BTreeSet::new();
    for path in [
        bundle.sources.manifest_path.as_deref(),
        bundle.sources.plan_manifest_path.as_deref(),
        bundle.sources.report_path.as_deref(),
        bundle.sources.run_summary_path.as_deref(),
        bundle.sources.facts_path.as_deref(),
        bundle.sources.environment_path.as_deref(),
        bundle.sources.runtime_policy_path.as_deref(),
        bundle.sources.run_state_path.as_deref(),
        bundle.sources.executor_descriptor_path.as_deref(),
        bundle.sources.checkpoint_path.as_deref(),
        bundle.sources.failure_path.as_deref(),
        bundle.sources.artifact_inventory_path.as_deref(),
        bundle.sources.replay_manifest_path.as_deref(),
        bundle.sources.hash_ledger_path.as_deref(),
        bundle.sources.evidence_verification_path.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        required.insert(path.to_string());
    }
    required.extend(bundle.sources.telemetry_paths.iter().cloned());
    required.extend(bundle.artifacts.iter().map(|artifact| artifact.path.clone()));
    if base_dir.join("bundle_signature.json").exists() {
        required.insert("bundle_signature.json".to_string());
    }
    if base_dir.join("reviewer_challenges.jsonl").exists() {
        required.insert("reviewer_challenges.jsonl".to_string());
    }
    if base_dir.join("reviewer_challenges.latest.json").exists() {
        required.insert("reviewer_challenges.latest.json".to_string());
    }

    required
}

fn verify_artifact_inventory_contract(path: &Path) -> (bool, String) {
    let Ok((inventory, audit)) =
        bijux_dna_runtime::run_layout::read_supported_artifact_inventory(path)
    else {
        return (false, format!("artifact inventory missing at {}", path.display()));
    };
    let role_complete = inventory.artifacts.iter().all(|row| !row.role.trim().is_empty());
    let lineage_complete = inventory.artifacts.iter().all(|row| !row.input_lineage.is_empty());
    (
        role_complete && lineage_complete,
        if role_complete && lineage_complete {
            format!(
                "artifact inventory accepted via {} and lineage is complete",
                audit.from_schema_version
            )
        } else {
            format!(
                "artifact inventory accepted via {} but role or lineage detail is incomplete",
                audit.from_schema_version
            )
        },
    )
}

#[allow(clippy::too_many_lines)]
fn profile_requirements(
    bundle: &EvidenceBundleV1,
    profile: EvidenceBundleProfileV1,
) -> Vec<(String, bool, String)> {
    let required_paths = match profile {
        EvidenceBundleProfileV1::Draft => vec![
            ("manifest_path", bundle.sources.manifest_path.as_deref()),
            ("artifact_inventory_path", bundle.sources.artifact_inventory_path.as_deref()),
        ],
        EvidenceBundleProfileV1::Operational => vec![
            ("manifest_path", bundle.sources.manifest_path.as_deref()),
            ("plan_manifest_path", bundle.sources.plan_manifest_path.as_deref()),
            ("run_state_path", bundle.sources.run_state_path.as_deref()),
            ("runtime_policy_path", bundle.sources.runtime_policy_path.as_deref()),
            ("artifact_inventory_path", bundle.sources.artifact_inventory_path.as_deref()),
            ("evidence_verification_path", bundle.sources.evidence_verification_path.as_deref()),
        ],
        EvidenceBundleProfileV1::Certification => vec![
            ("manifest_path", bundle.sources.manifest_path.as_deref()),
            ("plan_manifest_path", bundle.sources.plan_manifest_path.as_deref()),
            ("run_state_path", bundle.sources.run_state_path.as_deref()),
            ("runtime_policy_path", bundle.sources.runtime_policy_path.as_deref()),
            ("report_path", bundle.sources.report_path.as_deref()),
            ("run_summary_path", bundle.sources.run_summary_path.as_deref()),
            ("artifact_inventory_path", bundle.sources.artifact_inventory_path.as_deref()),
            ("hash_ledger_path", bundle.sources.hash_ledger_path.as_deref()),
            ("evidence_verification_path", bundle.sources.evidence_verification_path.as_deref()),
        ],
        EvidenceBundleProfileV1::Publication | EvidenceBundleProfileV1::PublicationStrict => vec![
            ("manifest_path", bundle.sources.manifest_path.as_deref()),
            ("plan_manifest_path", bundle.sources.plan_manifest_path.as_deref()),
            ("report_path", bundle.sources.report_path.as_deref()),
            ("run_summary_path", bundle.sources.run_summary_path.as_deref()),
            ("facts_path", bundle.sources.facts_path.as_deref()),
            ("artifact_inventory_path", bundle.sources.artifact_inventory_path.as_deref()),
            ("hash_ledger_path", bundle.sources.hash_ledger_path.as_deref()),
            ("evidence_verification_path", bundle.sources.evidence_verification_path.as_deref()),
        ],
        EvidenceBundleProfileV1::CollaboratorRedacted => vec![
            ("manifest_path", bundle.sources.manifest_path.as_deref()),
            ("report_path", bundle.sources.report_path.as_deref()),
            ("run_summary_path", bundle.sources.run_summary_path.as_deref()),
            ("artifact_inventory_path", bundle.sources.artifact_inventory_path.as_deref()),
            ("hash_ledger_path", bundle.sources.hash_ledger_path.as_deref()),
        ],
        EvidenceBundleProfileV1::ArchiveRetention => vec![
            ("manifest_path", bundle.sources.manifest_path.as_deref()),
            ("plan_manifest_path", bundle.sources.plan_manifest_path.as_deref()),
            ("report_path", bundle.sources.report_path.as_deref()),
            ("run_summary_path", bundle.sources.run_summary_path.as_deref()),
            ("facts_path", bundle.sources.facts_path.as_deref()),
            ("artifact_inventory_path", bundle.sources.artifact_inventory_path.as_deref()),
            ("replay_manifest_path", bundle.sources.replay_manifest_path.as_deref()),
            ("hash_ledger_path", bundle.sources.hash_ledger_path.as_deref()),
            ("evidence_verification_path", bundle.sources.evidence_verification_path.as_deref()),
        ],
    };
    required_paths
        .into_iter()
        .map(|(check_id, value)| {
            let ok = value.is_some();
            (
                format!("{check_id}_required"),
                ok,
                if ok {
                    format!("{check_id} is present for {profile:?} evidence bundles")
                } else {
                    format!("{check_id} is required for {profile:?} evidence bundles")
                },
            )
        })
        .chain(std::iter::once((
            "telemetry_required".to_string(),
            !bundle.sources.telemetry_paths.is_empty(),
            if bundle.sources.telemetry_paths.is_empty() {
                format!("telemetry paths are required for {profile:?} evidence bundles")
            } else {
                format!("telemetry paths are present for {profile:?} evidence bundles")
            },
        )))
        .chain(matches!(profile, EvidenceBundleProfileV1::PublicationStrict).then_some((
            "strict_publication_methods_summary_required".to_string(),
            bundle.methods_summary.is_some(),
            if bundle.methods_summary.is_some() {
                "methods summary present for strict publication bundle".to_string()
            } else {
                "methods summary is required for strict publication bundle".to_string()
            },
        )))
        .chain(matches!(profile, EvidenceBundleProfileV1::PublicationStrict).then_some((
            "strict_publication_citations_required".to_string(),
            !bundle.citations.is_empty(),
            if bundle.citations.is_empty() {
                "citations are required for strict publication bundle".to_string()
            } else {
                "citations present for strict publication bundle".to_string()
            },
        )))
        .chain(matches!(profile, EvidenceBundleProfileV1::PublicationStrict).then_some((
            "strict_publication_no_gaps".to_string(),
            bundle.health.gaps.is_empty(),
            if bundle.health.gaps.is_empty() {
                "strict publication bundle has no evidence gaps".to_string()
            } else {
                "strict publication bundle must not contain evidence gaps".to_string()
            },
        )))
        .collect()
}

fn tolerated_gap_codes(profile: EvidenceBundleProfileV1) -> Vec<String> {
    match profile {
        EvidenceBundleProfileV1::Draft => vec![
            "missing_hash_ledger".to_string(),
            "missing_report".to_string(),
            "missing_run_summary".to_string(),
        ],
        EvidenceBundleProfileV1::Operational => vec!["missing_hash_ledger".to_string()],
        EvidenceBundleProfileV1::Certification
        | EvidenceBundleProfileV1::Publication
        | EvidenceBundleProfileV1::PublicationStrict
        | EvidenceBundleProfileV1::CollaboratorRedacted
        | EvidenceBundleProfileV1::ArchiveRetention => Vec::new(),
    }
}

fn verify_hash_ledger_contract(base_dir: &Path, path: &Path) -> (bool, String) {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return (false, format!("hash ledger missing at {}", path.display()));
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return (false, format!("hash ledger is not valid json at {}", path.display()));
    };
    let schema_ok = value.get("schema_version").and_then(serde_json::Value::as_str)
        == Some("bijux.hash_ledger.v1");
    let entries = value.get("entries").and_then(serde_json::Value::as_array);
    let entries_ok = entries.is_some_and(|rows| {
        let mut previous = None;
        rows.iter().all(|row| {
            let Some(relative_path) = row.get("path").and_then(serde_json::Value::as_str) else {
                return false;
            };
            let Some(expected_hash) = row.get("sha256").and_then(serde_json::Value::as_str) else {
                return false;
            };
            let recorded_previous = row
                .get("previous_entry_sha256")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string);
            if recorded_previous != previous {
                return false;
            }
            let full = base_dir.join(relative_path);
            let Ok(actual_hash) = bijux_dna_infra::hash_file_sha256(&full) else {
                return false;
            };
            previous = Some(expected_hash.to_string());
            actual_hash == expected_hash
        })
    });
    (
        schema_ok && entries_ok,
        if schema_ok && entries_ok {
            "hash ledger ordering and file hashes verified".to_string()
        } else {
            "hash ledger failed schema or chained hash verification".to_string()
        },
    )
}

fn verify_report_completeness(path: &Path) -> (bool, String) {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return (false, format!("report missing at {}", path.display()));
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return (false, format!("report is not valid json at {}", path.display()));
    };
    let completeness = value.get("completeness");
    let missing_metrics_empty = completeness
        .and_then(|value| value.get("missing_metrics"))
        .and_then(serde_json::Value::as_array)
        .is_none_or(Vec::is_empty);
    let missing_reports_empty = completeness
        .and_then(|value| value.get("missing_reports"))
        .and_then(serde_json::Value::as_array)
        .is_none_or(Vec::is_empty);
    (
        missing_metrics_empty && missing_reports_empty,
        if missing_metrics_empty && missing_reports_empty {
            "report completeness is governed and empty of missing fields".to_string()
        } else {
            "report completeness declares missing metrics or reports".to_string()
        },
    )
}

fn verify_advisory_and_enforced_consistency(
    manifest_path: &Path,
    run_state_path: &Path,
) -> (bool, String) {
    let manifest = std::fs::read_to_string(manifest_path)
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok());
    let run_state = std::fs::read_to_string(run_state_path)
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok());
    let manifest_mode =
        manifest.as_ref().and_then(|value| value.get("mode")).and_then(serde_json::Value::as_str);
    let state_mode =
        run_state.as_ref().and_then(|value| value.get("mode")).and_then(serde_json::Value::as_str);
    let ok = manifest_mode == state_mode;
    (
        ok,
        if ok {
            "manifest mode and run-state mode are consistent".to_string()
        } else {
            "manifest mode and run-state mode disagree".to_string()
        },
    )
}
