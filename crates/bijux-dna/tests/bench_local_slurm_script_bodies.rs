#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

#[test]
fn bench_local_validate_slurm_script_bodies_refuses_placeholder_job_bodies() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("slurm-dry-run");
    std::fs::create_dir_all(&root).expect("create root");
    let script_path = root.join("fake-placeholder.sbatch");
    std::fs::write(
        &script_path,
        "#!/usr/bin/env bash\nset -euo pipefail\necho execute placeholder job\nrc=0\n",
    )
    .expect("write fake script");
    let report_path = temp.path().join("no-placeholder-report.json");

    let output = run_cli(&[
        "bench",
        "local",
        "validate-slurm-script-bodies",
        "--root",
        root.to_str().expect("root str"),
        "--output",
        report_path.to_str().expect("report str"),
        "--json",
    ]);

    assert!(!output.status.success(), "command should fail on placeholder script");

    let payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).expect("read report"))
            .expect("parse report");
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(false));
    assert_eq!(payload.get("script_count").and_then(serde_json::Value::as_u64), Some(1));
    let scripts =
        payload.get("scripts").and_then(serde_json::Value::as_array).expect("scripts array");
    let findings = scripts[0]
        .get("findings")
        .and_then(serde_json::Value::as_array)
        .expect("findings array")
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert!(
        findings.iter().any(|finding| finding.contains("placeholder")),
        "findings must include placeholder detection"
    );
    assert!(
        findings.iter().any(|finding| finding.contains("echo execute")),
        "findings must include fake echo detection"
    );
    assert!(
        findings.iter().any(|finding| finding.contains("unconditional `rc=0`")),
        "findings must include unconditional rc=0 detection"
    );
    assert!(
        findings.iter().any(|finding| finding.contains("missing `bijux-dna` command")),
        "findings must include missing bijux-dna detection"
    );
}
