#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::process::Command;

fn cargo_target_dir(root: &std::path::Path) -> std::path::PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target"))
}

#[test]
fn policy__contracts__cli_release_help_snapshot_policy__release_help_matches_snapshot_exactly() {
    let root = support::workspace_root();
    let snapshot_path = root.join("docs/cli/release_help_snapshot.txt");
    let expected = std::fs::read_to_string(&snapshot_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", snapshot_path.display()));

    let output = Command::new(cargo_target_dir(&root).join("debug/bijux"))
        .arg("dna")
        .arg("--help")
        .current_dir(&root)
        .output()
        .unwrap_or_else(|err| panic!("run debug help: {err}"));
    assert!(output.status.success(), "debug help command failed");
    let actual = String::from_utf8(output.stdout).expect("debug help must be valid UTF-8");

    assert_eq!(
        actual.trim(),
        expected.trim(),
        "docs/cli/release_help_snapshot.txt is stale. Regenerate with: target/debug/bijux dna --help > docs/cli/release_help_snapshot.txt"
    );
}
