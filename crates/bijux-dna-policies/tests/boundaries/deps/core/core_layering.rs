#![allow(non_snake_case)]
use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use walkdir::WalkDir;

#[path = "core_layering/source_admission.rs"]
mod source_admission;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

struct CoreLayeringSources {
    foundation: PathBuf,
    contract: PathBuf,
}

impl CoreLayeringSources {
    fn from_repository(root: &Path) -> Self {
        let core = root.join("crates").join("bijux-dna-core").join("src");
        Self { foundation: core.join("foundation"), contract: core.join("contract") }
    }
}

#[derive(Debug)]
struct LayeringObservation {
    files: Vec<PathBuf>,
    offenders: Vec<String>,
}

fn collect_rs_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let kind = std::fs::symlink_metadata(dir)
        .with_context(|| format!("inspect required core source {}", dir.display()))?
        .file_type();
    anyhow::ensure!(
        kind.is_dir() && !kind.is_symlink(),
        "required core source must be an unredirected directory: {}",
        dir.display()
    );
    let mut files = Vec::new();
    for entry in WalkDir::new(dir) {
        let entry =
            entry.with_context(|| format!("walk required core source {}", dir.display()))?;
        anyhow::ensure!(
            !entry.file_type().is_symlink(),
            "required core source must not redirect: {}",
            entry.path().display()
        );
        if entry.file_type().is_dir()
            || entry.path().extension().and_then(|extension| extension.to_str()) != Some("rs")
        {
            continue;
        }
        anyhow::ensure!(
            entry.file_type().is_file(),
            "required core Rust source must be a regular file: {}",
            entry.path().display()
        );
        files.push(entry.into_path());
    }
    files.sort();
    anyhow::ensure!(
        !files.is_empty(),
        "required core source contains no Rust files: {}",
        dir.display()
    );
    Ok(files)
}

fn inspect_layering(sources: &CoreLayeringSources) -> Result<LayeringObservation> {
    let mut observation = LayeringObservation { files: Vec::new(), offenders: Vec::new() };
    for (directory, forbidden) in [
        (&sources.foundation, &["crate::contract", "crate::metrics::registry"][..]),
        (&sources.contract, &["crate::foundation::invariants"][..]),
    ] {
        for file in collect_rs_files(directory)? {
            let content = std::fs::read_to_string(&file)
                .with_context(|| format!("read {}", file.display()))?;
            for needle in forbidden {
                if content.contains(needle) {
                    observation.offenders.push(format!(
                        "{} imports forbidden {}",
                        file.display(),
                        needle
                    ));
                }
            }
            observation.files.push(file);
        }
    }
    Ok(observation)
}

#[test]
fn policy__boundaries__core_layering__core_layering_is_enforced() {
    let sources = CoreLayeringSources::from_repository(&workspace_root());
    let observation = inspect_layering(&sources)
        .unwrap_or_else(|error| panic!("core layering acquisition failed: {error:#}"));
    writeln!(
        std::io::stderr().lock(),
        "core layering observation: foundation={} contract={} inspected Rust sources={}",
        sources.foundation.display(),
        sources.contract.display(),
        observation.files.len()
    )
    .unwrap_or_else(|error| panic!("record core layering observation: {error}"));
    bijux_dna_policies::policy_assert!(
        observation.offenders.is_empty(),
        "core layering violations:\n{}",
        observation.offenders.join("\n")
    );
}
