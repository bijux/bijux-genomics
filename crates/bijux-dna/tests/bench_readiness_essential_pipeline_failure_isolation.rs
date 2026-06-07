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
fn bench_readiness_essential_pipeline_failure_isolation_tracks_failed_and_blocked_rows() {
    let payload = run_cli_json(&[
        "bench",
        "readiness",
        "render-essential-pipeline-failure-isolation",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.essential_pipeline_failure_isolation.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/essential-pipeline-failure-isolation.json")
    );
    assert_eq!(
        payload.get("simulation_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-fake-runs/pipelines/essential-failure-isolation")
    );
    assert_eq!(payload.get("pipeline_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(93));
    assert_eq!(payload.get("completed_node_count").and_then(serde_json::Value::as_u64), Some(91));
    assert_eq!(payload.get("failed_node_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("blocked_node_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        payload.get("seeded_failed_node_id").and_then(serde_json::Value::as_str),
        Some("relatedness-segments-vcf::vcf.ibd")
    );
    assert_eq!(payload.get("passes_behavior_test"), Some(&serde_json::Value::Bool(true)));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 93);

    let failed_row = rows
        .iter()
        .find(|row| {
            row.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("relatedness-segments-vcf")
                && row.get("node_id").and_then(serde_json::Value::as_str) == Some("vcf.ibd")
        })
        .expect("failed row");
    assert_eq!(
        failed_row.get("execution_state").and_then(serde_json::Value::as_str),
        Some("failed")
    );
    assert_eq!(
        failed_row.get("reason").and_then(serde_json::Value::as_str),
        Some("injected_stage_failure")
    );
    assert_eq!(failed_row.get("exit_code").and_then(serde_json::Value::as_i64), Some(17));
    assert_eq!(failed_row.get("outputs_present"), Some(&serde_json::Value::Bool(false)));

    let blocked_row = rows
        .iter()
        .find(|row| {
            row.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("relatedness-segments-vcf")
                && row.get("node_id").and_then(serde_json::Value::as_str) == Some("vcf.demography")
        })
        .expect("blocked row");
    assert_eq!(
        blocked_row.get("execution_state").and_then(serde_json::Value::as_str),
        Some("blocked")
    );
    assert_eq!(
        blocked_row.get("reason").and_then(serde_json::Value::as_str),
        Some("failed_dependency_blocked")
    );

    let continued_row = rows
        .iter()
        .find(|row| {
            row.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("relatedness-segments-vcf")
                && row.get("node_id").and_then(serde_json::Value::as_str) == Some("vcf.roh")
        })
        .expect("continued row");
    assert_eq!(
        continued_row.get("execution_state").and_then(serde_json::Value::as_str),
        Some("completed")
    );
    assert_eq!(
        continued_row.get("unrelated_branch_continues"),
        Some(&serde_json::Value::Bool(true))
    );
}
