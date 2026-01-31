use assert_cmd::Command;

#[test]
fn analyze_help_lists_modes() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bijux"));
    cmd.args(["analyze", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("summary"))
        .stdout(predicates::str::contains("compare"))
        .stdout(predicates::str::contains("rank"))
        .stdout(predicates::str::contains("report"));
}
