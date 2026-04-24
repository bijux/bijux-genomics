#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

#[test]
fn slow__policy__contracts__toy_golden_runs_policy__toy_inputs_and_goldens_are_deterministic() {
    let root = support::workspace_root();
    let checksum = root.join("assets/toy/core-v1/CHECKSUMS.sha256");
    if !checksum.exists() {
        eprintln!("skip toy golden deterministic check; missing {}", checksum.display());
        return;
    }
    let status = std::process::Command::new("cargo")
        .arg("run")
        .arg("-q")
        .arg("-p")
        .arg("bijux-dna-dev")
        .arg("--")
        .arg("test")
        .arg("run")
        .arg("toy-runs")
        .arg("--")
        .arg("check")
        .arg("--profile")
        .arg("all")
        .arg("--out")
        .arg(root.join("artifacts/toy_policy_check"))
        .status()
        .unwrap_or_else(|err| panic!("run bijux-dna-dev test run toy-runs check: {err}"));
    assert!(status.success(), "toy golden check failed");
}

#[test]
fn slow__policy__contracts__toy_golden_runs_policy__golden_refresh_requires_accept_flag() {
    let root = support::workspace_root();
    let status = std::process::Command::new("cargo")
        .arg("run")
        .arg("-q")
        .arg("-p")
        .arg("bijux-dna-dev")
        .arg("--")
        .arg("test")
        .arg("run")
        .arg("toy-runs")
        .arg("--")
        .arg("refresh")
        .arg("--profile")
        .arg("all")
        .arg("--out")
        .arg(root.join("artifacts/toy_policy_check_refresh"))
        .status()
        .unwrap_or_else(|err| {
            panic!("run bijux-dna-dev test run toy-runs refresh without accept: {err}")
        });
    assert!(!status.success(), "toy golden refresh must fail without --accept");
}

#[test]
fn policy__contracts__toy_golden_runs_policy__make_refresh_targets_use_assets_scripts_only() {
    let root = support::workspace_root();
    let mk = std::fs::read_to_string(root.join("makes/cargo.mk"))
        .unwrap_or_else(|err| panic!("read makes/cargo.mk: {err}"));
    assert!(
        mk.contains("cargo run -q -p bijux-dna-dev -- assets run refresh-toy")
            && mk.contains("cargo run -q -p bijux-dna-dev -- assets run refresh-golden"),
        "toy refresh targets must call the native assets control plane"
    );
}
