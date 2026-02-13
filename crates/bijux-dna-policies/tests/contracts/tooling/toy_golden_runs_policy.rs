#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__toy_golden_runs_policy__toy_inputs_and_goldens_are_deterministic() {
    let root = support::workspace_root();
    let status = std::process::Command::new("python3")
        .arg(root.join("scripts/test/toy_runs.py"))
        .arg("check")
        .arg("--profile")
        .arg("all")
        .arg("--out")
        .arg(root.join("artifacts/toy_policy_check"))
        .status()
        .unwrap_or_else(|err| panic!("run toy_runs.py check: {err}"));
    assert!(status.success(), "toy golden check failed");
}

#[test]
fn policy__contracts__toy_golden_runs_policy__golden_refresh_requires_accept_flag() {
    let root = support::workspace_root();
    let status = std::process::Command::new("python3")
        .arg(root.join("scripts/test/toy_runs.py"))
        .arg("refresh")
        .arg("--profile")
        .arg("all")
        .arg("--out")
        .arg(root.join("artifacts/toy_policy_check_refresh"))
        .status()
        .unwrap_or_else(|err| panic!("run toy_runs.py refresh without accept: {err}"));
    assert!(
        !status.success(),
        "toy golden refresh must fail without --accept"
    );
}
