#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::process::Command;

fn cargo_target_dir(root: &std::path::Path) -> std::path::PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map_or_else(|| root.join("artifacts/target"), std::path::PathBuf::from)
}

fn run_workspace_bijux_dna(
    root: &std::path::Path,
    args: &[&str],
    context: &str,
) -> std::process::Output {
    let debug_binary = cargo_target_dir(root).join("debug/bijux-dna");
    let mut command = if debug_binary.exists() {
        let mut command = Command::new(debug_binary);
        command.current_dir(root);
        command
    } else {
        let mut command = Command::new("cargo");
        command.current_dir(root).args([
            "run",
            "-q",
            "-p",
            "bijux-dna",
            "--bin",
            "bijux-dna",
            "--",
        ]);
        command
    };
    command.args(args).output().unwrap_or_else(|err| {
        panic!("{context}: {err}");
    })
}

#[test]
fn policy__contracts__cli_release_help_snapshot_policy__release_help_matches_snapshot_exactly() {
    let root = support::workspace_root();
    let snapshot_path = root.join("docs/cli/release_help_snapshot.txt");
    let expected = std::fs::read_to_string(&snapshot_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", snapshot_path.display()));

    let output = run_workspace_bijux_dna(&root, &["--help"], "run debug help");
    assert!(output.status.success(), "debug help command failed");
    let actual = String::from_utf8(output.stdout).expect("debug help must be valid UTF-8");

    assert_eq!(
        actual.trim(),
        expected.trim(),
        "docs/cli/release_help_snapshot.txt is stale. Regenerate with: cargo run -q -p bijux-dna -- --help > docs/cli/release_help_snapshot.txt"
    );
}
