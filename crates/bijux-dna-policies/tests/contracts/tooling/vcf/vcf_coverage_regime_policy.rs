#![allow(non_snake_case)]
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__vcf_coverage_regime_policy__coverage_regime_ssot_exists() {
    let root = repo_root();
    let path = root.join("configs/runtime/coverage_regimes.toml");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    bijux_dna_policies::policy_assert!(
        content.contains("[decision.coverage_regime]"),
        "coverage regime SSOT must define [decision.coverage_regime] in {}",
        path.display()
    );
    bijux_dna_policies::policy_assert!(
        content.contains("allowed_values = [\"gl\", \"pseudohaploid\", \"diploid\"]"),
        "coverage regime SSOT must enumerate allowed values gl/pseudohaploid/diploid in {}",
        path.display()
    );
    for profile in [
        "[decision.coverage_regime.profiles.adna_lowcov_capture]",
        "[decision.coverage_regime.profiles.adna_lowcov_shotgun]",
        "[decision.coverage_regime.profiles.modern_wgs_capture]",
        "[decision.coverage_regime.profiles.modern_wgs_shotgun]",
    ] {
        bijux_dna_policies::policy_assert!(
            content.contains(profile),
            "coverage regime SSOT missing profile block {profile} in {}",
            path.display()
        );
    }
}

#[test]
fn policy__contracts__vcf_coverage_regime_policy__vcf_profiles_require_coverage_decision_binding() {
    let root = repo_root();
    let profiles_dir = root.join("configs/runtime/profiles");
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&profiles_dir).expect("read runtime profiles dir") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.contains("vcf") {
            continue;
        }
        let content = std::fs::read_to_string(&path).expect("read profile");
        let has_binding = content.contains("coverage_decision = \"decision.coverage_regime\"");
        let has_gate = content.contains("calling_regime_from_decision = true");
        let has_profile = content.contains("regime_profile = ");
        if !(has_binding && has_gate && has_profile) {
            offenders.push(path.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "VCF runtime profiles must bind calling regime to decision.coverage_regime, set calling_regime_from_decision=true, and declare regime_profile:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__vcf_coverage_regime_policy__no_incompatible_lowcov_diploid_stage_mix() {
    let root = repo_root();
    let profiles_dir = root.join("configs/runtime/profiles");
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&profiles_dir).expect("read runtime profiles dir") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let content = std::fs::read_to_string(&path).expect("read profile");
        let is_lowcov = content.contains("regime_profile = \"adna_lowcov_capture\"")
            || content.contains("regime_profile = \"adna_lowcov_shotgun\"");
        let has_diploid_stage = content.contains("vcf.call_diploid");
        if is_lowcov && has_diploid_stage {
            offenders.push(path.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Low-coverage regime profiles cannot include vcf.call_diploid stage:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__vcf_coverage_regime_policy__vcf_examples_include_regime_observability_fields()
{
    let root = repo_root();
    let checks = [
        "examples/vcf/imputation-mini/golden/explain.json",
        "examples/vcf/imputation-mini/golden/report.json",
        "examples/vcf/downstream-demography-mini/golden/explain.json",
        "examples/vcf/downstream-demography-mini/golden/report.json",
    ];
    let mut offenders = Vec::new();
    for rel in checks {
        let path = root.join(rel);
        let content = std::fs::read_to_string(&path).expect("read example golden");
        let has_selected = content.contains("\"selected\"");
        let has_thresholds = content.contains("\"thresholds_used\"");
        let has_observed = content.contains("\"observed_coverage_stats\"");
        if !(has_selected && has_thresholds && has_observed) {
            offenders.push(path.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "VCF explain/report goldens must include coverage_regime selected/thresholds_used/observed_coverage_stats:\n{}",
        offenders.join("\n")
    );
}
