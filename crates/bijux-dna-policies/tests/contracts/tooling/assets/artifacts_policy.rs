#![allow(non_snake_case)]
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__artifacts_policy__artifacts_are_gitignored_and_untracked() {
    let root = repo_root();
    let gitignore = std::fs::read_to_string(root.join(".gitignore")).unwrap_or_default();
    let ignores_artifacts = gitignore.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == "artifacts/" || trimmed == "/artifacts/" || trimmed == "artifacts/**"
    });
    bijux_dna_policies::policy_assert!(
        ignores_artifacts,
        ".gitignore must ignore artifacts/ for isolate and CI outputs"
    );

    let output = Command::new("git")
        .arg("ls-files")
        .arg("artifacts")
        .current_dir(&root)
        .output()
        .expect("run git ls-files artifacts");
    let tracked = String::from_utf8_lossy(&output.stdout).trim().to_string();
    bijux_dna_policies::policy_assert!(
        tracked.is_empty(),
        "no files under artifacts/ may be tracked by git:\n{}",
        tracked
    );
}
