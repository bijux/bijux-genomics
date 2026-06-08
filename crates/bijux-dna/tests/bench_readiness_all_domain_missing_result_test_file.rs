#![allow(clippy::expect_used)]

use std::path::PathBuf;
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

fn run_cli_json_with_repo_root(args: &[&str]) -> (PathBuf, serde_json::Value) {
    let repo_root = support::repo_root().expect("repo root");
    let output = run_cli(args);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (repo_root, serde_json::from_slice(&output.stdout).expect("parse stdout as json"))
}

#[test]
fn bench_readiness_all_domain_missing_result_test_writes_governed_report_and_fixture_tree() {
    let (repo_root, payload) = run_cli_json_with_repo_root(&[
        "bench",
        "readiness",
        "render-all-domain-missing-result-test",
        "--json",
    ]);

    let report_path = repo_root.join("benchmarks/readiness/missing-result-test-all-domains.json");
    assert!(report_path.is_file(), "all-domain missing-result report must exist");

    let persisted: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).expect("read missing-result report"))
            .expect("parse missing-result report");
    assert_eq!(
        persisted.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/missing-result-test-all-domains.json")
    );
    assert_eq!(
        persisted.get("missing_result_row_count").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(persisted.get("expected_row_count").and_then(serde_json::Value::as_u64), Some(125));
    assert_eq!(
        persisted.get("present_result_row_count").and_then(serde_json::Value::as_u64),
        Some(122)
    );

    let fixture_root =
        repo_root.join("runs/bench/readiness-probes/all-domains/missing-result-test");
    assert!(fixture_root.is_dir(), "all-domain missing-result fixture root must exist");
    assert!(
        fixture_root.join("manifest.json").is_file(),
        "all-domain fake-run manifest must exist"
    );

    let removed_manifest_paths = payload
        .get("removed_manifest_paths")
        .and_then(serde_json::Value::as_array)
        .expect("removed manifest paths");
    assert_eq!(removed_manifest_paths.len(), 3);
    for path in removed_manifest_paths {
        let absolute = repo_root.join(path.as_str().expect("manifest path"));
        assert!(!absolute.exists(), "removed manifest `{}` must stay absent", absolute.display());
    }

    let rows = persisted.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    let present_manifest = rows
        .iter()
        .find(|row| row.get("result_status").and_then(serde_json::Value::as_str) == Some("present"))
        .and_then(|row| row.get("audit_manifest_path").and_then(serde_json::Value::as_str))
        .expect("present manifest path");
    assert!(
        repo_root.join(present_manifest).is_file(),
        "present rows must keep a realized manifest path"
    );
}
