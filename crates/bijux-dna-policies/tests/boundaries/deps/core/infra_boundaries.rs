#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]
use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use std::io::Write as _;
use std::path::{Path, PathBuf};

#[path = "infra_boundaries/source_admission.rs"]
mod source_admission;
#[path = "../../../support/rust_source_intake.rs"]
mod source_intake;

#[test]
fn policy__boundaries__infra_boundaries__infra_has_no_domain_semantics() {
    let metadata = MetadataCommand::new().exec().expect("cargo metadata");
    let infra = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == "bijux-dna-infra")
        .expect("bijux-dna-infra missing");
    let banned = [
        "bijux-dna-domain-bam",
        "bijux-dna-domain-fastq",
        "bijux-dna-stages-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-planner-bam",
        "bijux-dna-planner-fastq",
    ];
    let mut offenders = Vec::new();
    for dep in &infra.dependencies {
        if banned.contains(&dep.name.as_str()) {
            offenders.push(dep.name.clone());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-infra must not depend on domain/stage/planner crates.\n\
Remove semantic dependencies; infra should be utility-only.\n\
See docs/40-policies/STYLE.md.\n\
Offenders: {offenders:?}"
    );
}

#[derive(Debug)]
struct SourceObservation {
    files: Vec<PathBuf>,
    offenders: Vec<String>,
}

fn infra_source_directory(root: &Path) -> PathBuf {
    root.join("crates/bijux-dna-infra/src")
}

fn inspect_infra(source: &Path) -> Result<SourceObservation> {
    let files = source_intake::collect_rs_files(source)?;
    let mut offenders = Vec::new();
    for file in &files {
        let content =
            std::fs::read_to_string(file).with_context(|| format!("read {}", file.display()))?;
        if content.contains("StageId::new(\"") || content.contains("ToolId::new(\"") {
            offenders.push(file.display().to_string());
        }
    }
    Ok(SourceObservation { files, offenders })
}

#[test]
fn policy__boundaries__infra_boundaries__no_id_catalog_literals_in_infra() {
    let root = bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"));
    let source = infra_source_directory(&root);
    let observation = inspect_infra(&source)
        .unwrap_or_else(|error| panic!("infra source acquisition failed: {error:#}"));
    writeln!(
        std::io::stderr().lock(),
        "infra source observation: root={} inspected Rust sources={}",
        source.display(),
        observation.files.len()
    )
    .unwrap_or_else(|error| panic!("record infra source observation: {error}"));
    bijux_dna_policies::policy_assert!(
        observation.offenders.is_empty(),
        "bijux-dna-infra must not define StageId/ToolId catalogs or literals.\n\
Move semantics to domain/planner/core.\n\
See docs/40-policies/STYLE.md.\n\
Offenders:\n{}",
        observation.offenders.join("\n")
    );
}
