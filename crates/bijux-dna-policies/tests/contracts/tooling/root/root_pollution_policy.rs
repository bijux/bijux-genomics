#![allow(non_snake_case)]
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__root_pollution_policy__tracked_root_outputs_are_forbidden() {
    let root = repo_root();
    let output =
        Command::new("git").arg("ls-files").current_dir(&root).output().expect("run git ls-files");
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
        "tracked root pollution paths are forbidden (use artifacts/ or target/):\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__root_pollution_policy__target_and_artifacts_are_gitignored() {
    let root = repo_root();
    let gitignore =
        std::fs::read_to_string(root.join(".gitignore")).expect("read workspace .gitignore");
    let has_target = gitignore.contains("/target/") || gitignore.contains("**/target/");
    let has_artifacts = gitignore.contains("/artifacts/");
    bijux_dna_policies::policy_assert!(
        has_target && has_artifacts,
        ".gitignore must ignore target/ and artifacts/ to keep root clean"
    );
}
