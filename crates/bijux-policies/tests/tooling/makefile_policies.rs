#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn policy__tooling__makefile_policies__only_root_makefile_exists() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let root_makefile = root.join("Makefile.toml");
    if root_makefile.exists() {
        offenders.push(root_makefile.display().to_string());
    }
    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if entry.file_type().is_file() && entry.file_name() == "Makefile.toml" {
            offenders.push(entry.path().display().to_string());
        }
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "per-crate Makefile.toml files are not allowed: {:?}",
        offenders
    );
}

#[test]
fn policy__tooling__makefile_policies__root_makefile_is_single_source() {
    let root = workspace_root();
    let makefile = root.join("Makefile");
    let content = std::fs::read_to_string(&makefile).expect("read Makefile");
    bijux_policies::policy_assert!(
        content.contains("include makefiles/cargo.mk"),
        "Makefile must include makefiles/cargo.mk"
    );
    let forbidden_targets = [
        "lint:",
        "test:",
        "test-slow:",
        "test-e2e:",
        "audit:",
        "bench:",
    ];
    let offenders: Vec<&str> = forbidden_targets
        .iter()
        .copied()
        .filter(|target| content.contains(target))
        .collect();
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "root Makefile must not duplicate cargo-make tasks: {:?}",
        offenders
    );
}
