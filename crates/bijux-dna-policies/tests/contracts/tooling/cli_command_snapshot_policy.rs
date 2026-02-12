#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::process::Command;

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

    let output = Command::new(root.join("target/debug/bijux-dna"))
        .arg("--help")
        .current_dir(&root)
        .output()
        .unwrap_or_else(|err| panic!("run 'bijux dna --help' via shim binary: {err}"));

    assert!(
        output.status.success(),
        "failed to run dna help command: status={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("dna help output must be valid UTF-8");
    assert_eq!(
        normalize_whitespace(&actual),
        normalize_whitespace(&expected),
        "docs/cli/command_snapshot.txt is stale. Regenerate with: target/debug/bijux-dna --help | sed -E 's/[[:space:]]+/ /g; s/^ //; s/ $//' > docs/cli/command_snapshot.txt"
    );
}
