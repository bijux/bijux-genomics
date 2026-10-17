#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

use crate::support;

fn run_cli_and_read_report(args: &[&str], report_path: &str) -> (std::process::Output, String) {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _repo_lock =
        support::RepoProcessLock::acquire("benchmark-readiness-mutators").expect("repo lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(support::cli_binary_path())
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli");
    let report = std::fs::read_to_string(repo_root.join(report_path)).expect("read report");
    (output, report)
}

#[test]
fn bench_local_validate_vcf_stage_catalog_ready_writes_governed_json_file() {
    let report_path =
        "artifacts/benchmark-smoke/readiness/local-ready/VCF_STAGE_CATALOG_READY.json";
    let (output, raw) = run_cli_and_read_report(
        &["bench", "local", "validate-vcf-stage-catalog-ready"],
        report_path,
    );
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), report_path);

    let parsed: serde_json::Value = serde_json::from_str(&raw).expect("parse report");

    assert_eq!(
        parsed.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_stage_catalog_ready.v1")
    );
    assert_eq!(parsed.get("checked_goal_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(parsed.get("failed_goal_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(parsed.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let checks = parsed.get("checks").and_then(serde_json::Value::as_array).expect("checks");
    assert_eq!(checks.len(), 9);
    assert!(checks.iter().all(|check| {
        check.get("ok").and_then(serde_json::Value::as_bool) == Some(true)
            && check
                .get("output_path")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
    }));

    let goal_ids = checks
        .iter()
        .map(|check| check.get("goal_id").and_then(serde_json::Value::as_u64).expect("goal id"))
        .collect::<Vec<_>>();
    assert_eq!(goal_ids, vec![201, 202, 203, 204, 205, 206, 207, 208, 209]);
}
