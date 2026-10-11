#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

fn normalize_help(input: &str) -> String {
    input.lines().map(str::trim_end).collect::<Vec<_>>().join("\n")
}

fn run_workspace_bijux_dna(
    root: &std::path::Path,
    args: &[&str],
    context: &str,
) -> std::process::Output {
    let mut command = support::workspace_binary_command(root, "bijux-dna", "bijux-dna")
        .unwrap_or_else(|error| panic!("{context}: {error}"));
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
        normalize_help(&actual),
        normalize_help(&expected),
        "docs/cli/release_help_snapshot.txt is stale. Regenerate with: cargo run -q -p bijux-dna -- --help > docs/cli/release_help_snapshot.txt"
    );
}
