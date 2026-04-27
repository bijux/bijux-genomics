#![allow(non_snake_case)]
use std::path::PathBuf;

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__makefile_policies__only_root_makefile_exists() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let root_makefile = root.join("Makefile.toml");
    if root_makefile.exists() {
        offenders.push(root_makefile.display().to_string());
    }
    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() && entry.file_name() == "Makefile.toml" {
            offenders.push(entry.path().display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "per-crate Makefile.toml files are not allowed: {:?}",
        offenders
    );
}

#[test]
fn policy__contracts__makefile_policies__root_makefile_is_single_source() {
    let root = workspace_root();
    let makefile = root.join("Makefile");
    let content = std::fs::read_to_string(&makefile).expect("read Makefile");
    bijux_dna_policies::policy_assert!(
        content.contains("include makes/root.mk"),
        "Makefile must include makes/root.mk as the canonical command surface"
    );
    let forbidden_targets = ["lint:", "test:", "test-slow:", "test-e2e:", "audit:", "bench:"];
    let offenders: Vec<&str> =
        forbidden_targets.iter().copied().filter(|target| content.contains(target)).collect();
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "root Makefile must not define command targets directly: {:?}",
        offenders
    );
}
