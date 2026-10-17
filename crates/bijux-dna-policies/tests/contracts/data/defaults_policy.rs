#![allow(non_snake_case)]
use anyhow::{Context, Result};
use bijux_dna_pipelines::registry::PipelineRegistry;
use std::io::Write as _;
use std::path::{Path, PathBuf};

#[path = "defaults_policy/source_admission.rs"]
mod source_admission;
#[path = "../../support/rust_source_intake.rs"]
mod source_intake;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[derive(Debug)]
struct SourceObservation {
    files: Vec<PathBuf>,
    offenders: Vec<String>,
}

fn default_source_roots(root: &Path) -> [PathBuf; 4] {
    [
        root.join("crates/bijux-dna-api/src"),
        root.join("crates/bijux-dna/src"),
        root.join("crates/bijux-dna-stages-fastq/src"),
        root.join("crates/bijux-dna-stages-bam/src"),
    ]
}

fn inspect_default_sources(targets: &[PathBuf]) -> Result<SourceObservation> {
    let regex_default = regex::Regex::new(r"\b[A-Za-z0-9_]*Params::default\b").unwrap();
    let regex_default_call = regex::Regex::new(r"Default::default\(\)").unwrap();
    let mut observation = SourceObservation { files: Vec::new(), offenders: Vec::new() };
    for target in targets {
        for file in source_intake::collect_rs_files(target)? {
            if file.to_string_lossy().contains("/tests/") {
                continue;
            }
            let content = std::fs::read_to_string(&file)
                .with_context(|| format!("read {}", file.display()))?;
            if regex_default.is_match(&content)
                || (regex_default_call.is_match(&content) && content.contains("Params"))
            {
                observation.offenders.push(file.display().to_string());
            }
            observation.files.push(file);
        }
    }
    Ok(observation)
}

#[test]
fn policy__contracts__defaults_policy__params_defaults_live_in_pipelines_only() {
    let targets = default_source_roots(&workspace_root());
    let observation = inspect_default_sources(&targets)
        .unwrap_or_else(|error| panic!("parameter-default source acquisition failed: {error:#}"));
    writeln!(
        std::io::stderr().lock(),
        "parameter-default observation: roots={targets:?} inspected Rust sources={}",
        observation.files.len()
    )
    .unwrap_or_else(|error| panic!("record parameter-default observation: {error}"));
    bijux_dna_policies::policy_assert!(
        observation.offenders.is_empty(),
        "param defaults must be defined in bijux-dna-pipelines only:\n{}",
        observation.offenders.join("\n")
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
