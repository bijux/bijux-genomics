use assert_cmd::Command;

#[test]
fn cli_reports_invalid_subcommand_with_hint() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bijux"));
    cmd.args(["fastq", "trm"]);
    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("similar subcommand exists"));
}

#[test]
fn cli_errors_on_missing_required_bench_args() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bijux"));
    cmd.args(["bench", "fastq", "validate", "--sample-id", "s1"]);
    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("required"));
}

#[test]
fn cli_exits_nonzero_on_missing_subcommand() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bijux"));
    cmd.args(["env"]);
    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicates::str::contains("subcommand"));
}
