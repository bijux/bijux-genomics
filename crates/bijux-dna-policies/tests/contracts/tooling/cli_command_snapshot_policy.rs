#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::process::Command;

fn cargo_target_dir(root: &std::path::Path) -> std::path::PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target"))
}

fn normalize_whitespace(text: &str) -> String {
    text.lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

#[test]
fn policy__contracts__cli_command_snapshot_policy__dna_help_matches_snapshot() {
    let root = support::workspace_root();
    let snapshot_path = root.join("docs/cli/command_snapshot.txt");
    let expected = std::fs::read_to_string(&snapshot_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", snapshot_path.display()));

    let output = Command::new(cargo_target_dir(&root).join("debug/bijux"))
        .arg("--help")
        .current_dir(&root)
        .output()
        .unwrap_or_else(|err| panic!("run 'bijux --help' via workspace binary: {err}"));

    assert!(
        output.status.success(),
        "failed to run root help command: status={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("root help output must be valid UTF-8");
    assert_eq!(
        normalize_whitespace(&actual),
        normalize_whitespace(&expected),
        "docs/cli/command_snapshot.txt is stale. Regenerate with: target/debug/bijux --help | sed -E 's/[[:space:]]+/ /g; s/^ //; s/ $//' > docs/cli/command_snapshot.txt"
    );
}
