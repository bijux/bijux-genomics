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
        .args(["bench", "readiness", "render-vcf-adapters-ready", "--json"])
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
fn bench_readiness_vcf_adapters_ready_reports_governed_pass_state() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_adapters_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/VCF_ADAPTERS_READY.json")
    );
    assert_eq!(payload.get("checked_goal_count").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(payload.get("passed_goal_count").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(payload.get("failed_goal_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("failing_goal_ids").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(0)
    );
    assert_eq!(
        payload.get("benchmark_ready_pair_count").and_then(serde_json::Value::as_u64),
        Some(20)
    );
    assert_eq!(
        payload.get("adapter_complete_pair_count").and_then(serde_json::Value::as_u64),
        Some(20)
    );
    assert_eq!(
        payload.get("output_complete_pair_count").and_then(serde_json::Value::as_u64),
        Some(20)
    );
    assert_eq!(
        payload.get("rendered_command_pair_count").and_then(serde_json::Value::as_u64),
        Some(20)
    );
    assert_eq!(payload.get("ok"), Some(&serde_json::Value::Bool(true)));

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks array");
    assert_eq!(checks.len(), 15);
    assert!(checks.iter().all(|check| check.get("ok") == Some(&serde_json::Value::Bool(true))));

    let matrix_check = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(234))
        .expect("goal 234 check");
    assert!(
        matrix_check
            .get("detail")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| detail.contains("matrix and registry")),
        "goal 234 detail must keep matrix/registry agreement explicit"
    );

    let output_check = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(242))
        .expect("goal 242 check");
    assert_eq!(
        output_check.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf-adapter-output-coverage.tsv")
    );

    let completeness_check = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(245))
        .expect("goal 245 check");
    assert!(
        completeness_check
            .get("detail")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| detail.contains("benchmark-ready VCF pairs")),
        "goal 245 detail must keep the cross-surface pair validation explicit"
    );
}
