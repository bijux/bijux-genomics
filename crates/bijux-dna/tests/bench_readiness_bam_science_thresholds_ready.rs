#![allow(clippy::expect_used, clippy::too_many_lines)]

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
fn bench_readiness_bam_science_thresholds_ready_reports_governed_metric_contracts() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-bam-science-thresholds-ready", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_science_thresholds_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam/BAM_SCIENCE_THRESHOLDS_READY.json")
    );
    assert_eq!(payload.get("comparable_stage_count").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(payload.get("stage_row_count").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(payload.get("governed_metric_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(
        payload.get("threshold_declared_stage_count").and_then(serde_json::Value::as_u64),
        Some(15)
    );
    assert_eq!(
        payload.get("missing_threshold_stage_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("threshold_declared_metric_count").and_then(serde_json::Value::as_u64),
        Some(51)
    );
    assert_eq!(
        payload.get("missing_threshold_metric_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 15);

    let damage = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.damage"))
        .expect("damage row");
    assert_eq!(
        damage.get("threshold_status").and_then(serde_json::Value::as_str),
        Some("declared")
    );
    assert_eq!(damage.get("metric_count").and_then(serde_json::Value::as_u64), Some(3));
    assert!(damage.get("metrics").and_then(serde_json::Value::as_array).is_some_and(|metrics| {
        metrics.iter().any(|metric| {
            metric.get("metric_name").and_then(serde_json::Value::as_str) == Some("damage_signal")
                && metric.get("pass_direction").and_then(serde_json::Value::as_str)
                    == Some("exact_match")
                && metric.get("tolerance_kind").and_then(serde_json::Value::as_str)
                    == Some("exact_match")
                && metric.get("tolerance_value").and_then(serde_json::Value::as_f64) == Some(0.0)
        })
    }));

    let validate = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.validate"))
        .expect("validate row");
    assert_eq!(validate.get("metric_count").and_then(serde_json::Value::as_u64), Some(3));
    assert!(validate.get("metrics").and_then(serde_json::Value::as_array).is_some_and(|metrics| {
        metrics.iter().any(|metric| {
            metric.get("metric_name").and_then(serde_json::Value::as_str)
                == Some("validation_errors")
                && metric.get("pass_direction").and_then(serde_json::Value::as_str)
                    == Some("structured_match")
                && metric.get("tolerance_kind").and_then(serde_json::Value::as_str)
                    == Some("normalized_set_overlap")
                && metric.get("tolerance_value").and_then(serde_json::Value::as_f64) == Some(1.0)
                && metric.get("insufficiency_policy").and_then(serde_json::Value::as_str)
                    == Some("refuse_stage_comparison")
        })
    }));
}
