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

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let output = run_cli(args);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn bench_local_validate_all_domain_slurm_script_bodies_reports_clean_governed_tree() {
    let payload =
        run_cli_json(&["bench", "local", "validate-all-domain-slurm-script-bodies", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_slurm_script_bodies.v1")
    );
    assert_eq!(
        payload.get("root_path").and_then(serde_json::Value::as_str),
        Some("target/slurm-dry-run/all-domains")
    );
    assert_eq!(
        payload.get("report_path").and_then(serde_json::Value::as_str),
        Some("target/slurm-dry-run/all-domains/no-placeholder-report.json")
    );
    assert_eq!(payload.get("script_count").and_then(serde_json::Value::as_u64), Some(213));
    assert_eq!(payload.get("findings_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let scripts =
        payload.get("scripts").and_then(serde_json::Value::as_array).expect("scripts array");
    assert_eq!(scripts.len(), 213);
    assert!(scripts.iter().all(|entry| {
        entry.get("ok").and_then(serde_json::Value::as_bool) == Some(true)
            && entry.get("has_bijux_dna_command").and_then(serde_json::Value::as_bool) == Some(true)
    }));
}
