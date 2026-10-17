use std::path::Path;

use anyhow::{ensure, Context as _, Result};
use bijux_dna_analyze::exports::{
    validate_evidence_bundle_profile, verify_profile_bundle, EvidenceProfileBundleV1,
};

pub(super) fn require_live_profile_admission(path: &Path) -> Result<()> {
    let original = std::fs::read(path)?;
    let original_permissions = std::fs::metadata(path)?.permissions();
    let produced: EvidenceProfileBundleV1 = serde_json::from_slice(&original)?;
    ensure!(
        validate_evidence_bundle_profile(&produced.evidence_bundle, produced.profile).ok,
        "real produced strict profile must satisfy pure policy"
    );
    ensure!(verify_profile_bundle(path)?.verified, "real persisted profile must verify");
    ensure!(
        produced.evidence_verification.verified && produced.profile_validation.ok,
        "causal control requires actually produced successful observations"
    );
    let mut changed = produced.clone();
    changed.evidence_bundle.sources.report_path = None;
    ensure!(
        !validate_evidence_bundle_profile(&changed.evidence_bundle, changed.profile).ok,
        "actual strict profile policy must refuse the missing source declaration"
    );
    std::fs::write(path, serde_json::to_vec_pretty(&changed)?)?;
    let semantic = verify_profile_bundle(path);
    std::fs::write(path, &original)?;
    ensure!(verify_profile_bundle(path)?.verified, "restored semantic control must verify");

    let report_name = produced
        .evidence_bundle
        .sources
        .report_path
        .as_deref()
        .context("real strict fixture must bind its report")?;
    ensure!(report_name == "report.json", "owned fixture report path changed");
    let root = path.parent().context("profile fixture parent")?;
    let report_path = root.join(report_name);
    let saved_report = root.join("report-source-preserved.json");
    ensure!(!saved_report.exists(), "source preservation path must be fresh");
    let report_bytes = std::fs::read(&report_path)?;
    let report_permissions = std::fs::metadata(&report_path)?.permissions();
    let bound = produced
        .required_files
        .iter()
        .find(|row| row.path == report_name)
        .context("real producer must include its report in required files")?;
    ensure!(
        bijux_dna_infra::hash_file_sha256(&report_path)? == bound.sha256,
        "real required report digest must match before acquisition negative"
    );
    let mut omitted = produced;
    omitted.required_files.clear();
    std::fs::write(path, serde_json::to_vec_pretty(&omitted)?)?;
    std::fs::rename(&report_path, &saved_report)?;
    let membership = verify_profile_bundle(path);
    std::fs::rename(&saved_report, &report_path)?;
    std::fs::write(path, &original)?;
    ensure!(
        std::fs::read(&report_path)? == report_bytes
            && std::fs::metadata(&report_path)?.permissions() == report_permissions,
        "required source must be restored before evaluating causal result"
    );
    ensure!(
        std::fs::read(path)? == original
            && std::fs::metadata(path)?.permissions() == original_permissions,
        "profile source must be restored before evaluating causal result"
    );
    ensure!(verify_profile_bundle(path)?.verified, "restored genuine profile must verify");
    eprintln!("ANALYZE_PROFILE_REOBSERVATION_CONTROL_PASS: original/restored valid; pure_changed_policy=false; semantic={semantic:?}; membership={membership:?}");
    let semantic = semantic.context("typed valid profile must return a verification outcome")?;
    let membership =
        membership.context("missing-source profile must return a verification outcome")?;
    let mut failures = Vec::new();
    if semantic.verified || semantic.profile_valid {
        failures.push("cached profile policy");
    }
    if membership.verified {
        failures.push("self-declared required membership");
    }
    ensure!(
        failures.is_empty(),
        "retained verifier admitted changed strict inputs using stored green: {failures:?}"
    );
    Ok(())
}

use bijux_dna_analyze::exports::EvidenceProfileBundleVerificationV1;

fn observe_profile_counterexamples(
    strict_path: &Path,
    collaborator_path: &Path,
    report_path: &Path,
    produced: EvidenceProfileBundleV1,
    mut collaborator: EvidenceProfileBundleV1,
    collaborator_bytes: &[u8],
) -> Result<(EvidenceProfileBundleVerificationV1, EvidenceProfileBundleVerificationV1)> {
    let binding = collaborator.required_files.first().cloned().context("produced binding")?;
    collaborator.required_files.push(binding);
    std::fs::write(collaborator_path, serde_json::to_vec_pretty(&collaborator)?)?;
    let duplicate = verify_profile_bundle(collaborator_path)?;
    std::fs::write(collaborator_path, collaborator_bytes)?;
    ensure!(
        verify_profile_bundle(collaborator_path)?.verified,
        "restored redacted control must verify"
    );

    let report_name = produced
        .evidence_bundle
        .sources
        .report_path
        .clone()
        .context("strict report declaration")?;
    ensure!(
        !produced.evidence_bundle.artifacts.iter().any(|artifact| artifact.path == report_name),
        "semantic control must not contain a second artifact digest for the report"
    );
    let mut changed_report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(report_path)?)?;
    changed_report
        .get_mut("completeness")
        .and_then(serde_json::Value::as_object_mut)
        .context("actual produced report completeness")?
        .insert(
            "missing_metrics".to_string(),
            serde_json::json!(["retained-source-missing-metric"]),
        );
    std::fs::write(report_path, serde_json::to_vec_pretty(&changed_report)?)?;
    let mut rebound = produced;
    ensure!(
        rebound.evidence_verification.verified && rebound.profile_validation.ok,
        "stored successful controls must remain intact"
    );
    rebound
        .required_files
        .iter_mut()
        .find(|row| row.path == report_name)
        .context("produced report digest binding")?
        .sha256 = bijux_dna_infra::hash_file_sha256(report_path)?;
    std::fs::write(strict_path, serde_json::to_vec_pretty(&rebound)?)?;
    Ok((duplicate, verify_profile_bundle(strict_path)?))
}

fn restore_profile_inputs(inputs: Vec<(&Path, Vec<u8>, std::fs::Permissions)>) -> Result<()> {
    let mut failures = Vec::new();
    for (path, bytes, permissions) in inputs {
        let restored = (|| -> Result<()> {
            std::fs::write(path, &bytes)?;
            std::fs::set_permissions(path, permissions.clone())?;
            ensure!(
                std::fs::read(path)? == bytes
                    && std::fs::metadata(path)?.permissions() == permissions,
                "profile inputs must restore before verdict evaluation"
            );
            Ok(())
        })();
        if let Err(error) = restored {
            failures.push(format!("{}: {error:#}", path.display()));
        }
    }
    ensure!(failures.is_empty(), "profile input restoration failed: {failures:?}");
    Ok(())
}

pub(super) fn require_current_profile_evidence(
    strict_path: &Path,
    collaborator_path: &Path,
) -> Result<()> {
    let strict_bytes = std::fs::read(strict_path)?;
    let collaborator_bytes = std::fs::read(collaborator_path)?;
    let produced: EvidenceProfileBundleV1 = serde_json::from_slice(&strict_bytes)?;
    let collaborator: EvidenceProfileBundleV1 = serde_json::from_slice(&collaborator_bytes)?;
    ensure!(
        verify_profile_bundle(collaborator_path)?.verified,
        "real produced collaborator-redacted profile must verify before mutation"
    );
    ensure!(verify_profile_bundle(strict_path)?.verified, "real strict control must verify");
    let report_name = produced
        .evidence_bundle
        .sources
        .report_path
        .as_deref()
        .context("strict report declaration")?;
    let report_path = strict_path.parent().context("strict profile root")?.join(report_name);
    let mut preserved = Vec::new();
    for path in [strict_path, collaborator_path, report_path.as_path()] {
        preserved.push((path, std::fs::read(path)?, std::fs::metadata(path)?.permissions()));
    }
    let observations = observe_profile_counterexamples(
        strict_path,
        collaborator_path,
        &report_path,
        produced,
        collaborator,
        &collaborator_bytes,
    );
    restore_profile_inputs(preserved)?;
    let (duplicate, semantic) = observations?;
    ensure!(
        verify_profile_bundle(collaborator_path)?.verified,
        "restored actual redacted control must verify"
    );
    ensure!(
        verify_profile_bundle(strict_path)?.verified,
        "restored actual strict control must verify"
    );
    eprintln!(
        "CURRENT_PROFILE_INPUT_CONTROLS_PASS: redacted/restored=true; duplicate={duplicate:?}; semantic_rebound={semantic:?}; physical_bytes_modes_restored=true"
    );
    ensure!(
        duplicate.hash_mismatches.is_empty() && duplicate.missing_paths.is_empty(),
        "duplicate binding control must retain every actual file/hash"
    );
    ensure!(!duplicate.verified, "an exact repeated required binding must not qualify");
    ensure!(
        semantic.hash_mismatches.is_empty() && semantic.missing_paths.is_empty(),
        "semantic control preserves every path and deliberately rebinds hashes"
    );
    ensure!(semantic.profile_valid, "pure retained profile policy remains positive");
    ensure!(
        !semantic.verified && !semantic.evidence_verified,
        "rebound digests cannot qualify current report-declared missing evidence"
    );
    Ok(())
}
