#![allow(non_snake_case)]
use anyhow::{Context, Result};
use std::io::Write as _;
use std::path::{Path, PathBuf};
#[path = "no_repo_tree_snapshots/source_admission.rs"]
mod source_admission;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[derive(Debug)]
struct SourceObservation {
    files: Vec<PathBuf>,
    offenders: Vec<String>,
}

fn snapshot_directory(root: &Path) -> PathBuf {
    root.join("crates/bijux-dna-policies/tests/snapshots")
}

fn inspect_snapshots(directory: &Path) -> Result<SourceObservation> {
    let kind = std::fs::symlink_metadata(directory)
        .with_context(|| format!("inspect required snapshot namespace {}", directory.display()))?
        .file_type();
    anyhow::ensure!(
        kind.is_dir() && !kind.is_symlink(),
        "required snapshot namespace must be an unredirected directory: {}",
        directory.display()
    );
    let mut observation = SourceObservation { files: Vec::new(), offenders: Vec::new() };
    let entries = std::fs::read_dir(directory)
        .with_context(|| format!("read required snapshot namespace {}", directory.display()))?;
    for entry in entries {
        let path = entry
            .with_context(|| {
                format!("enumerate required snapshot namespace {}", directory.display())
            })?
            .path();
        observation.files.push(path.clone());
        if path.extension().and_then(|extension| extension.to_str()) != Some("snap") {
            continue;
        }
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
        if name.contains("tree_contract") {
            observation.offenders.push(path.display().to_string());
        }
    }
    observation.files.sort();
    observation.offenders.sort();
    Ok(observation)
}

#[test]
fn policy__boundaries__no_repo_tree_snapshots__forbid_tree_contract_snapshots() {
    let directory = snapshot_directory(&workspace_root());
    let observation = inspect_snapshots(&directory)
        .unwrap_or_else(|error| panic!("snapshot namespace acquisition failed: {error:#}"));
    writeln!(
        std::io::stderr().lock(),
        "snapshot namespace observation: root={} observed entries={}",
        directory.display(),
        observation.files.len()
    )
    .unwrap_or_else(|error| panic!("record snapshot namespace observation: {error}"));
    bijux_dna_policies::policy_assert!(
        observation.offenders.is_empty(),
        "repo tree snapshots are not allowed; keep only API surface or schema snapshots:\n{}",
        observation.offenders.join("\n")
    );
}
