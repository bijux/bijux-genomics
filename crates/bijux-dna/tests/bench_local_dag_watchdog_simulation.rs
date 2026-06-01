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
fn bench_local_dag_watchdog_simulation_writes_no_global_wait_report() {
    let payload = run_cli_json(&["bench", "local", "simulate-dag-watchdog", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_dag_watchdog_simulation.v1")
    );
    assert_eq!(
        payload.get("scenario").and_then(serde_json::Value::as_str),
        Some("no_global_wait")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/pipelines/local/fastq-core-preprocess.toml")
    );
    assert_eq!(
        payload.get("dag_report_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/fastq-core-preprocess.json")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/dag-sim/no-global-wait.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("fastq-core-preprocess")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(
        payload
            .get("slow_branch_stage_id")
            .and_then(serde_json::Value::as_str),
        Some("fastq.profile_read_lengths")
    );
    assert_eq!(
        payload
            .get("slow_branch_finish_second")
            .and_then(serde_json::Value::as_u64),
        Some(13)
    );
    assert_eq!(
        payload
            .get("no_global_wait_proven")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let ready_nodes = payload
        .get("ready_while_slow_branch_running_stage_ids")
        .and_then(serde_json::Value::as_array)
        .expect("ready_while_slow_branch_running_stage_ids array");
    assert!(
        ready_nodes
            .iter()
            .any(|value| value.as_str() == Some("fastq.trim_reads")),
        "trim_reads must be reported as ready while the slow branch is still running"
    );
    assert!(
        ready_nodes
            .iter()
            .any(|value| value.as_str() == Some("fastq.filter_reads")),
        "filter_reads must be reported as ready while the slow branch is still running"
    );
}
