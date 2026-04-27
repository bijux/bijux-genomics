#![allow(non_snake_case)]
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__root_migration_guard_policy__new_top_level_dirs_are_blocked_with_guidance() {
    let root = repo_root();
    let allowed = [
        "assets",
        "bin",
        "configs",
        "containers",
        "crates",
        "docs",
        "domain",
        "examples",
        "makes",
        "science",
        "artifacts",
        "target",
    ];
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&root).expect("read root") {
        let entry = entry.expect("read entry");
        if !entry.file_type().expect("file type").is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name.starts_with("target") {
            continue;
        }
        if !allowed.contains(&name.as_str()) {
            offenders.push(name);
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "new top-level directory detected: {:?}\nMove to crates/ (code), configs/ (settings), assets/ (data), science/ (traceability specs and compiled science state), or bijux-dna-dev/native control-plane surfaces (automation).",
        offenders,
    );
}
