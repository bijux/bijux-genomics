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
    assert_eq!(payload.get("scenario").and_then(serde_json::Value::as_str), Some("no_global_wait"));
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/configs/pipelines/local/fastq-core-preprocess.toml")
    );
    assert_eq!(
        payload.get("dag_report_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/local-ready/pipeline-dag/fastq-core-preprocess.json")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/local-ready/dag-sim/no-global-wait.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("fastq-core-preprocess")
    );
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(
        payload.get("slow_branch_stage_id").and_then(serde_json::Value::as_str),
        Some("fastq.profile_read_lengths")
    );
    assert_eq!(
        payload.get("slow_branch_finish_second").and_then(serde_json::Value::as_u64),
        Some(13)
    );
    assert_eq!(
        payload.get("no_global_wait_proven").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let ready_nodes = payload
        .get("ready_while_slow_branch_running_stage_ids")
        .and_then(serde_json::Value::as_array)
        .expect("ready_while_slow_branch_running_stage_ids array");
    assert!(
        ready_nodes.iter().any(|value| value.as_str() == Some("fastq.trim_reads")),
        "trim_reads must be reported as ready while the slow branch is still running"
    );
    assert!(
        ready_nodes.iter().any(|value| value.as_str() == Some("fastq.filter_reads")),
        "filter_reads must be reported as ready while the slow branch is still running"
    );
}

#[test]
fn bench_local_dag_watchdog_simulation_writes_failure_isolation_report() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "simulate-dag-watchdog",
        "--scenario",
        "failure-isolation",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_dag_watchdog_simulation.v1")
    );
    assert_eq!(
        payload.get("scenario").and_then(serde_json::Value::as_str),
        Some("failure_isolation")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/local-ready/dag-sim/failure-isolation.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("fastq-core-preprocess")
    );
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        payload.get("failed_sample_id").and_then(serde_json::Value::as_str),
        Some("sample_alpha")
    );
    assert_eq!(
        payload.get("failed_stage_id").and_then(serde_json::Value::as_str),
        Some("fastq.detect_adapters")
    );
    assert_eq!(payload.get("failure_second").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(
        payload.get("failure_isolation_proven").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let continued_nodes = payload
        .get("continued_unrelated_node_ids")
        .and_then(serde_json::Value::as_array)
        .expect("continued_unrelated_node_ids array");
    assert!(
        continued_nodes.iter().any(|value| value.as_str() == Some("sample_beta::fastq.trim_reads")),
        "the unaffected sample must continue trimming after the injected failure"
    );
    assert!(
        continued_nodes.iter().any(|value| value.as_str() == Some("sample_beta::fastq.report_qc")),
        "the unaffected sample must continue through downstream QC after the injected failure"
    );

    let blocked_nodes = payload
        .get("blocked_node_ids")
        .and_then(serde_json::Value::as_array)
        .expect("blocked_node_ids array");
    assert!(
        blocked_nodes.iter().any(|value| value.as_str() == Some("sample_alpha::fastq.trim_reads")),
        "the failed sample must block only its own dependent downstream work"
    );
}

#[test]
fn bench_local_dag_watchdog_simulation_writes_partial_resume_report() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "simulate-dag-watchdog",
        "--scenario",
        "partial-resume",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_dag_watchdog_simulation.v1")
    );
    assert_eq!(payload.get("scenario").and_then(serde_json::Value::as_str), Some("partial_resume"));
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/local-ready/dag-sim/partial-resume.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("fastq-core-preprocess")
    );
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        payload.get("partial_resume_proven").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let reused_nodes = payload
        .get("reused_valid_node_ids")
        .and_then(serde_json::Value::as_array)
        .expect("reused_valid_node_ids array");
    assert!(
        reused_nodes.iter().any(|value| value.as_str() == Some("fastq.detect_adapters")),
        "valid completed nodes must be reported as reused"
    );

    let invalid_nodes = payload
        .get("invalid_node_ids")
        .and_then(serde_json::Value::as_array)
        .expect("invalid_node_ids array");
    assert_eq!(invalid_nodes.len(), 1);
    assert_eq!(
        invalid_nodes[0].as_str(),
        Some("fastq.trim_reads"),
        "the governed invalid node must be replanned"
    );

    let missing_nodes = payload
        .get("missing_node_ids")
        .and_then(serde_json::Value::as_array)
        .expect("missing_node_ids array");
    assert!(
        missing_nodes.iter().any(|value| value.as_str() == Some("fastq.filter_reads")),
        "downstream missing work must stay explicit in the report"
    );

    let planned_nodes = payload
        .get("planned_node_ids")
        .and_then(serde_json::Value::as_array)
        .expect("planned_node_ids array");
    assert!(
        planned_nodes.iter().any(|value| value.as_str() == Some("fastq.trim_reads")),
        "the invalid node must appear in the planned set"
    );
    assert!(
        planned_nodes.iter().any(|value| value.as_str() == Some("fastq.report_qc")),
        "missing downstream work must appear in the planned set"
    );
}

#[test]
fn bench_local_dag_watchdog_simulation_writes_completion_rules_report() {
    let payload = run_cli_json(&[
        "bench",
        "local",
        "simulate-dag-watchdog",
        "--scenario",
        "completion-rules",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_dag_watchdog_simulation.v1")
    );
    assert_eq!(
        payload.get("scenario").and_then(serde_json::Value::as_str),
        Some("completion_rules")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/local-ready/dag-sim/completion-rules.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("fastq-core-preprocess")
    );
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(
        payload.get("completion_check_stage_id").and_then(serde_json::Value::as_str),
        Some("fastq.filter_reads")
    );
    assert_eq!(
        payload.get("completion_rules_proven").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let completion_checks = payload
        .get("completion_checks")
        .and_then(serde_json::Value::as_array)
        .expect("completion_checks array");
    assert!(
        completion_checks.iter().any(|value| {
            value.get("case_id").and_then(serde_json::Value::as_str)
                == Some("zero_exit_outputs_only")
                && value.get("exit_code").and_then(serde_json::Value::as_i64) == Some(0)
                && value.get("declared_outputs_exist").and_then(serde_json::Value::as_bool)
                    == Some(true)
                && value.get("result_manifest_exists").and_then(serde_json::Value::as_bool)
                    == Some(false)
                && value.get("complete").and_then(serde_json::Value::as_bool) == Some(false)
        }),
        "zero exit with outputs alone must remain incomplete"
    );
    assert!(
        completion_checks.iter().any(|value| {
            value.get("case_id").and_then(serde_json::Value::as_str)
                == Some("zero_exit_outputs_and_manifest")
                && value.get("declared_outputs_exist").and_then(serde_json::Value::as_bool)
                    == Some(true)
                && value.get("result_manifest_exists").and_then(serde_json::Value::as_bool)
                    == Some(true)
                && value.get("complete").and_then(serde_json::Value::as_bool) == Some(true)
        }),
        "only the all-requirements case should be complete"
    );
}
