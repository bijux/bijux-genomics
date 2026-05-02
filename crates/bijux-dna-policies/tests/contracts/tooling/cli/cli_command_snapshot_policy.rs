#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::process::Command;

fn run_workspace_bijux_dna(
    root: &std::path::Path,
    args: &[&str],
    context: &str,
) -> std::process::Output {
    let mut command = Command::new("cargo");
    command.current_dir(root).args(["run", "-q", "-p", "bijux-dna", "--bin", "bijux-dna", "--"]);
    command.args(args).output().unwrap_or_else(|err| {
        panic!("{context}: {err}");
    })
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
fn slow__policy__contracts__cli_command_snapshot_policy__dna_help_matches_snapshot() {
    let root = support::workspace_root();
    let snapshot_path = root.join("docs/cli/command_snapshot.txt");
    let expected = std::fs::read_to_string(&snapshot_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", snapshot_path.display()));

    let output =
        run_workspace_bijux_dna(&root, &["--help"], "run 'bijux-dna --help' via workspace binary");

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
        "docs/cli/command_snapshot.txt is stale. Regenerate with: cargo run -q -p bijux-dna -- --help | sed -E 's/[[:space:]]+/ /g; s/^ //; s/ $//' > docs/cli/command_snapshot.txt"
    );
}
