#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("resolve repo root")
        .to_path_buf()
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
        if !(has_binding && has_gate) {
            offenders.push(path.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "VCF runtime profiles must bind calling regime to decision.coverage_regime and set calling_regime_from_decision=true:\n{}",
        offenders.join("\n")
    );
}
