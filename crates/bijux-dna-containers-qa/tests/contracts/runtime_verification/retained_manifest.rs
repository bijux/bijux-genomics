//! Current and baseline reuse must retain the semantic evidence of the real produced manifest.
use std::path::Path;

use anyhow::{ensure, Context as _, Result};
use bijux_dna_containers_qa::api::{
    execute_runtime_verification, ApptainerSmokeManifest, RuntimeVerificationPublication,
    RuntimeVerificationRequest,
};

pub(super) fn require_live_manifest_semantics(
    request: &RuntimeVerificationRequest,
    baseline_path: &Path,
) -> Result<()> {
    let manifest_path = request.evidence_root.join("manifests/example.json");
    let original = std::fs::read(&manifest_path)?;
    let original_permissions = std::fs::metadata(&manifest_path)?.permissions();
    let baseline_bytes = std::fs::read(baseline_path)?;
    let baseline_permissions = std::fs::metadata(baseline_path)?.permissions();
    let produced: ApptainerSmokeManifest = serde_json::from_slice(&original)?;
    ensure!(
        produced.status == "ok"
            && produced.version_matches_contract
            && produced.help_ok
            && !produced.probe_results.is_empty()
            && produced.probe_results.iter().all(|probe| probe.ok && !probe.timed_out),
        "actual public producer must supply successful current evidence"
    );
    let output_root = request.evidence_root.join("semantic-admission");
    std::fs::create_dir(&output_root)?;
    let current_control = execute_runtime_verification(
        request,
        &output_root.join("valid-current.json"),
        &output_root.join("valid-current.md"),
    )?;
    ensure!(
        current_control.passed == 1 && current_control.failed == 0,
        "real retained current manifest must admit before mutation"
    );
    let mut variants = Vec::new();
    let mut changed = produced.clone();
    changed.version_matches_contract = false;
    variants.push(("version-contradiction", changed));
    let mut changed = produced.clone();
    changed.help_ok = false;
    variants.push(("help-contradiction", changed));
    let mut changed = produced;
    changed.probe_results.clear();
    variants.push(("missing-probe-roster", changed));
    let mut admitted = Vec::new();
    for (label, changed) in variants {
        ensure!(changed.status == "ok", "causal variant preserves claimed status");
        std::fs::write(&manifest_path, serde_json::to_vec_pretty(&changed)?)?;
        let current = execute_runtime_verification(
            request,
            &output_root.join(format!("current-{label}.json")),
            &output_root.join(format!("current-{label}.md")),
        );
        if current.as_ref().is_ok_and(|result| result.passed != 0) {
            admitted.push(format!("current {label}"));
        }
        // Retain the real publication and update only the actual changed manifest's claimed digest.
        let mut baseline: RuntimeVerificationPublication = serde_json::from_slice(&baseline_bytes)?;
        let source = baseline
            .tools
            .get_mut("example")
            .context("produced baseline tool")?
            .manifest
            .as_mut()
            .context("produced baseline manifest identity")?;
        source.sha256 = bijux_dna_infra::hash_file_sha256(&manifest_path)?;
        let private_baseline = output_root.join(format!("baseline-source-{label}.json"));
        std::fs::write(&private_baseline, serde_json::to_vec_pretty(&baseline)?)?;
        let mut baseline_request = request.clone();
        baseline_request.baseline_publication_path = Some(private_baseline);
        baseline_request.evidence_root = output_root.join(format!("reused-{label}"));
        let reused = execute_runtime_verification(
            &baseline_request,
            &output_root.join(format!("reused-{label}.json")),
            &output_root.join(format!("reused-{label}.md")),
        );
        if reused.as_ref().is_ok_and(|result| result.passed != 0) {
            admitted.push(format!("baseline {label}"));
        }
        eprintln!(
            "RUNTIME_RETAINED_MANIFEST_OBSERVED: {label}; current={current:?}; baseline={reused:?}"
        );
        std::fs::write(&manifest_path, &original)?;
        ensure!(
            std::fs::read(&manifest_path)? == original
                && std::fs::metadata(&manifest_path)?.permissions() == original_permissions,
            "produced manifest restored before next observation"
        );
    }
    ensure!(
        std::fs::read(baseline_path)? == baseline_bytes
            && std::fs::metadata(baseline_path)?.permissions() == baseline_permissions,
        "original baseline publication bytes and mode unchanged"
    );
    let restored = execute_runtime_verification(
        request,
        &output_root.join("restored-current.json"),
        &output_root.join("restored-current.md"),
    )?;
    ensure!(
        restored.passed == 1 && restored.failed == 0,
        "restored actual producer remains admitted"
    );
    eprintln!("RUNTIME_RETAINED_MANIFEST_CONTROLS_PASS: original producer, exact baseline reuse, digest-consistent mutations and restored current control; admitted={admitted:?}");
    ensure!(
        admitted.is_empty(),
        "runtime reuse certified contradictory manifest semantics: {admitted:?}"
    );
    Ok(())
}
