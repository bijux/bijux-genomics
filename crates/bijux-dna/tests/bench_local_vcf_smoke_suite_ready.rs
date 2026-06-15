#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
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
        .args(args)
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
fn bench_local_validate_vcf_smoke_suite_ready_reports_governed_goal_slice() {
    let payload = run_cli_json(&["bench", "local", "validate-vcf-smoke-suite-ready", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_smoke_suite_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/VCF_SMOKE_SUITE_READY.json")
    );
    assert_eq!(payload.get("checked_goal_count").and_then(serde_json::Value::as_u64), Some(19));
    assert_eq!(payload.get("passed_goal_count").and_then(serde_json::Value::as_u64), Some(19));
    assert_eq!(payload.get("failed_goal_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let failing_goal_ids = payload
        .get("failing_goal_ids")
        .and_then(serde_json::Value::as_array)
        .expect("failing goal ids");
    assert!(failing_goal_ids.is_empty());

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks");
    assert_eq!(checks.len(), 19);
    assert!(checks
        .iter()
        .all(|check| check.get("ok").and_then(serde_json::Value::as_bool) == Some(true)));

    let goal_211 = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(211))
        .expect("goal 211 check");
    assert_eq!(goal_211.get("surface").and_then(serde_json::Value::as_str), Some("vcf.call smoke"));
    assert_eq!(
        goal_211.get("output_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.call/bcftools/calls.vcf.gz")
    );

    let goal_222 = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(222))
        .expect("goal 222 check");
    assert_eq!(
        goal_222.get("surface").and_then(serde_json::Value::as_str),
        Some("vcf.impute smoke")
    );
    assert!(goal_222
        .get("detail")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|detail| detail.contains("masked-truth")));

    let goal_229 = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(229))
        .expect("goal 229 check");
    assert_eq!(
        goal_229.get("output_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.demography/ibdne/demography.json")
    );
    assert!(goal_229
        .get("detail")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|detail| detail.contains("insufficient-data probe")));
}
