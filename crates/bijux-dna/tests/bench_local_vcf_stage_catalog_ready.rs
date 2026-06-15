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
fn bench_local_validate_vcf_stage_catalog_ready_reports_governed_goal_slice() {
    let payload = run_cli_json(&["bench", "local", "validate-vcf-stage-catalog-ready", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_stage_catalog_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/local-ready/VCF_STAGE_CATALOG_READY.json")
    );
    assert_eq!(payload.get("checked_goal_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(payload.get("passed_goal_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(payload.get("failed_goal_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let failing_goal_ids = payload
        .get("failing_goal_ids")
        .and_then(serde_json::Value::as_array)
        .expect("failing goal ids");
    assert!(failing_goal_ids.is_empty());

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks");
    assert_eq!(checks.len(), 9);
    assert!(checks
        .iter()
        .all(|check| check.get("ok").and_then(serde_json::Value::as_bool) == Some(true)));

    let goal_201 = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(201))
        .expect("goal 201 check");
    assert_eq!(
        goal_201.get("surface").and_then(serde_json::Value::as_str),
        Some("vcf stage catalog")
    );
    assert_eq!(
        goal_201.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/configs/local/vcf-stage-catalog.toml")
    );
    assert!(goal_201
        .get("detail")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|detail| detail.contains("20")));

    let goal_207 = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(207))
        .expect("goal 207 check");
    assert_eq!(
        goal_207.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/fixtures/vcf-mini-regeneration/manifest.json")
    );
    assert!(goal_207
        .get("detail")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|detail| detail.contains("matching governed counts")));

    let goal_209 = checks
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(209))
        .expect("goal 209 check");
    assert_eq!(
        goal_209.get("surface").and_then(serde_json::Value::as_str),
        Some("vcf no-empty-output gate")
    );
    assert_eq!(
        goal_209.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/local-ready/vcf/no-empty-output-check.json")
    );
    assert!(goal_209
        .get("detail")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|detail| detail.contains("declared outputs")));
}
