#![allow(clippy::expect_used)]

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
fn bench_local_vcf_demography_smoke_reports_main_run_and_probe() {
    let payload = run_cli_json(&["bench", "local", "run-vcf-demography-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_demography_smoke.v1")
    );
    assert_eq!(
        payload.get("command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-demography-smoke --tool-id ibdne")
    );
    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.demography"));
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("ibdne"));
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(
        payload.get("input_fixture_id").and_then(serde_json::Value::as_str),
        Some("vcf_mini_multisample_cohort")
    );
    assert_eq!(
        payload.get("output_root").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.demography/ibdne")
    );
    assert_eq!(
        payload.get("demography_json_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.demography/ibdne/demography.json")
    );
    assert_eq!(payload.get("method").and_then(serde_json::Value::as_str), Some("ibdne"));
    assert_eq!(payload.get("status").and_then(serde_json::Value::as_str), Some("complete"));
    assert_eq!(payload.get("insufficient_reason").and_then(serde_json::Value::as_str), None);
    let time_bins =
        payload.get("time_bins").and_then(serde_json::Value::as_array).expect("time_bins");
    let ne_estimates =
        payload.get("ne_estimates").and_then(serde_json::Value::as_array).expect("ne_estimates");
    assert!(!time_bins.is_empty(), "expected non-empty time bins");
    assert_eq!(time_bins.len(), ne_estimates.len());

    let probe = payload.get("insufficient_data_probe").expect("insufficient data probe");
    assert_eq!(probe.get("method").and_then(serde_json::Value::as_str), Some("ibdne"));
    assert_eq!(probe.get("status").and_then(serde_json::Value::as_str), Some("insufficient_data"));
    assert_eq!(
        probe.get("insufficient_reason").and_then(serde_json::Value::as_str),
        Some("not_enough_ibd_segments")
    );
    assert_eq!(probe.get("time_bins").and_then(serde_json::Value::as_array).map(Vec::len), Some(0));
    assert_eq!(
        probe.get("ne_estimates").and_then(serde_json::Value::as_array).map(Vec::len),
        Some(0)
    );

    let repo_root = support::repo_root().expect("repo root");
    let persisted_path = repo_root.join("target/local-smoke/vcf.demography/ibdne/demography.json");
    let persisted_raw = std::fs::read_to_string(&persisted_path).expect("read persisted report");
    let persisted: serde_json::Value = serde_json::from_str(&persisted_raw).expect("parse report");
    assert_eq!(
        persisted.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_demography_smoke.v1")
    );
}
