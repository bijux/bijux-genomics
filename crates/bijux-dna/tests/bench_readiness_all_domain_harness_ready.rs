#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json() -> serde_json::Value {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-all-domain-harness-ready", "--json"])
        .output()
        .expect("run cli");

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
fn bench_readiness_all_domain_harness_ready_reports_governed_pass_state() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_harness_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/ALL_DOMAIN_HARNESS_READY.json")
    );
    assert_eq!(payload.get("checked_goal_count").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(payload.get("passed_goal_count").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(payload.get("failed_goal_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("failing_goal_ids").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(0)
    );
    assert_eq!(payload.get("all_domain_stage_count").and_then(serde_json::Value::as_u64), Some(71));
    assert_eq!(
        payload.get("benchmark_ready_binding_count").and_then(serde_json::Value::as_u64),
        Some(120)
    );
    assert_eq!(
        payload.get("expected_result_row_count").and_then(serde_json::Value::as_u64),
        Some(120)
    );
    assert_eq!(
        payload.get("rendered_command_row_count").and_then(serde_json::Value::as_u64),
        Some(120)
    );
    assert_eq!(
        payload.get("output_declaration_row_count").and_then(serde_json::Value::as_u64),
        Some(120)
    );
    assert_eq!(payload.get("fake_run_result_count").and_then(serde_json::Value::as_u64), Some(120));
    assert_eq!(payload.get("fake_run_output_count").and_then(serde_json::Value::as_u64), Some(482));
    assert_eq!(
        payload.get("fake_failure_result_count").and_then(serde_json::Value::as_u64),
        Some(120)
    );
    assert_eq!(
        payload.get("fake_failure_output_count").and_then(serde_json::Value::as_u64),
        Some(482)
    );
    assert_eq!(payload.get("completion_row_count").and_then(serde_json::Value::as_u64), Some(120));
    assert_eq!(
        payload.get("parser_collector_row_count").and_then(serde_json::Value::as_u64),
        Some(123)
    );
    assert_eq!(
        payload.get("missing_result_row_count").and_then(serde_json::Value::as_u64),
        Some(120)
    );
    assert_eq!(payload.get("failure_class_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(
        payload.get("real_smoke_execution_count").and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(payload.get("ok"), Some(&serde_json::Value::Bool(true)));

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks array");
    assert_eq!(checks.len(), 12);
    assert!(checks.iter().all(|check| check.get("ok") == Some(&serde_json::Value::Bool(true))));

    let stage_inventory_check = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(278))
        .expect("goal 278 check");
    assert!(
        stage_inventory_check
            .get("detail")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| detail.contains("71-stage")),
        "goal 278 detail must keep the all-domain inventory explicit"
    );

    let real_smoke_check = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(289))
        .expect("goal 289 check");
    assert!(
        real_smoke_check
            .get("detail")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| detail.contains("bam-to-vcf bridge")),
        "goal 289 detail must keep the bridge execution explicit"
    );
}
