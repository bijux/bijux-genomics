#![allow(non_snake_case)]
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("resolve repo root")
        .to_path_buf()
}

#[test]
fn policy__contracts__root_pollution_policy__tracked_root_outputs_are_forbidden() {
    let root = repo_root();
    let output = Command::new("git")
        .arg("ls-files")
        .current_dir(&root)
        .output()
        .expect("run git ls-files");
    let files = String::from_utf8_lossy(&output.stdout);
    let mut offenders = Vec::new();
    for line in files.lines() {
        let rel = line.trim();
        if rel.is_empty() {
            continue;
        }
        if rel.starts_with("coverage/")
            || rel == "coverage"
            || rel.starts_with("target-")
            || rel.starts_with("target_isolate")
        {
            offenders.push(rel.to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "tracked root pollution paths are forbidden (use artifacts/isolates or target/):\n{}",
        offenders.join("\n")
    );
}
