#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

use bijux_dna_pipelines::registry::PipelineRegistry;
use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .map(|entry| entry.path().to_path_buf())
        .collect()
}

#[test]
fn policy__contracts__defaults_policy__params_defaults_live_in_pipelines_only() {
    let root = workspace_root();
    let targets = [
        root.join("../../../../bijux-dna-api/src"),
        root.join("../../../../bijux-dna/src"),
        root.join("../../../../bijux-dna-stages-fastq/src"),
        root.join("../../../../bijux-dna-stages-bam/src"),
    ];
    let regex_default = regex::Regex::new(r"\b[A-Za-z0-9_]*Params::default\b").unwrap();
    let regex_default_call = regex::Regex::new(r"Default::default\(\)").unwrap();
    let mut offenders = Vec::new();

    for target in targets {
        for file in collect_rs_files(&target) {
            if file.to_string_lossy().contains("/tests/") {
                continue;
            }
            let content = std::fs::read_to_string(&file).expect("read source");
            if regex_default.is_match(&content)
                || (regex_default_call.is_match(&content) && content.contains("Params"))
            {
                offenders.push(file.display().to_string());
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "param defaults must be defined in bijux-dna-pipelines only:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__defaults_policy__every_default_has_provenance() {
    let registry = PipelineRegistry::v1();
    let mut offenders = Vec::new();

    for profile in registry.list(true) {
        let ledger = profile.defaults_ledger();
        for stage in ledger.tools.keys() {
            match ledger.tool_provenance.get(stage) {
                Some(provenance)
                    if !provenance.rationale.trim().is_empty()
                        && !provenance.assumptions.is_empty()
                        && !provenance.comparability_implications.is_empty()
                        && !contains_unspecified(&provenance.rationale)
                        && !provenance.assumptions.iter().any(|v| contains_unspecified(v))
                        && !provenance
                            .comparability_implications
                            .iter()
                            .any(|v| contains_unspecified(v)) => {}
                _ => offenders.push(format!(
                    "{} missing tool provenance for {}",
                    profile.id.as_str(),
                    stage.as_str()
                )),
            }
        }
        for stage in ledger.params.keys() {
            match ledger.param_provenance.get(stage) {
                Some(provenance)
                    if !provenance.rationale.trim().is_empty()
                        && !provenance.assumptions.is_empty()
                        && !provenance.comparability_implications.is_empty()
                        && !contains_unspecified(&provenance.rationale)
                        && !provenance.assumptions.iter().any(|v| contains_unspecified(v))
                        && !provenance
                            .comparability_implications
                            .iter()
                            .any(|v| contains_unspecified(v)) => {}
                _ => offenders.push(format!(
                    "{} missing param provenance for {}",
                    profile.id.as_str(),
                    stage.as_str()
                )),
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "every defaulted tool/param must have provenance:\n{}",
        offenders.join("\n")
    );
}

fn contains_unspecified(value: &str) -> bool {
    value.trim().eq_ignore_ascii_case("unspecified")
}
