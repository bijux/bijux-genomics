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
fn plan_validate_all_local_pipelines_reports_benchmark_root_contract() {
    let payload = run_cli_json(&[
        "plan",
        "validate",
        "--benchmark-root",
        "benchmarks",
        "--all",
        "--strict",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_pipeline_dag_validation_set.v1")
    );
    assert_eq!(
        payload.get("benchmark_root").and_then(serde_json::Value::as_str),
        Some("benchmarks")
    );
    assert_eq!(
        payload.get("config_root").and_then(serde_json::Value::as_str),
        Some("benchmarks/configs/pipelines/local")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/all-benchmark-pipelines.json")
    );
    assert_eq!(payload.get("pipeline_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("valid_pipeline_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("all_valid").and_then(serde_json::Value::as_bool), Some(true));

    let pipelines =
        payload.get("pipelines").and_then(serde_json::Value::as_array).expect("pipelines array");
    assert_eq!(pipelines.len(), 20);
    assert!(
        pipelines.iter().any(|report| {
            report.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("core-germline-fastq-bam-vcf")
                && report.get("config_path").and_then(serde_json::Value::as_str)
                    == Some("benchmarks/configs/pipelines/local/core-germline-fastq-bam-vcf.toml")
        }),
        "all-pipeline validation must include the governed core germline benchmark pipeline"
    );
}
