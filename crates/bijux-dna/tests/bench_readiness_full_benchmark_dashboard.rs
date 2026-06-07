#![allow(clippy::expect_used)]

use std::collections::BTreeMap;
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
fn bench_readiness_full_benchmark_dashboard_tracks_governed_summary_counts() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-full-benchmark-dashboard", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.full_benchmark_dashboard.v1")
    );
    assert_eq!(
        payload.get("markdown_output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/FASTQ_BAM_VCF_BENCHMARK_DASHBOARD.md")
    );
    assert_eq!(
        payload.get("json_output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/FASTQ_BAM_VCF_BENCHMARK_DASHBOARD.json")
    );
    assert_eq!(
        payload.get("passes_behavior_test").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        payload.get("total_stages").and_then(serde_json::Value::as_u64),
        Some(71)
    );
    assert_eq!(
        payload.get("total_tools").and_then(serde_json::Value::as_u64),
        Some(64)
    );
    assert_eq!(
        payload.get("total_expected_jobs").and_then(serde_json::Value::as_u64),
        Some(120)
    );
    assert_eq!(
        payload.get("ready_jobs").and_then(serde_json::Value::as_u64),
        Some(117)
    );
    assert_eq!(
        payload.get("blocked_jobs").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(
        payload.get("missing_parsers").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("missing_adapters").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("missing_assets").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("failed_real_smokes").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload
            .get("explicit_unsupported_pairs")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let metrics = payload
        .get("metrics")
        .and_then(serde_json::Value::as_array)
        .expect("metrics array");
    assert_eq!(metrics.len(), 9);

    let counts = metrics
        .iter()
        .map(|metric| {
            let metric_id = metric
                .get("metric_id")
                .and_then(serde_json::Value::as_str)
                .expect("metric id")
                .to_string();
            let count = metric
                .get("count")
                .and_then(serde_json::Value::as_u64)
                .expect("metric count");
            (metric_id, count)
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(counts.get("total_stages").copied(), Some(71));
    assert_eq!(counts.get("total_tools").copied(), Some(64));
    assert_eq!(counts.get("total_expected_jobs").copied(), Some(120));
    assert_eq!(counts.get("ready_jobs").copied(), Some(117));
    assert_eq!(counts.get("blocked_jobs").copied(), Some(3));
    assert_eq!(counts.get("missing_parsers").copied(), Some(0));
    assert_eq!(counts.get("missing_adapters").copied(), Some(0));
    assert_eq!(counts.get("missing_assets").copied(), Some(0));
    assert_eq!(counts.get("failed_real_smokes").copied(), Some(0));
}
