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
fn plan_validate_local_pipeline_reports_core_germline_contract() {
    let payload = run_cli_json(&[
        "plan",
        "validate",
        "--id",
        "core-germline-fastq-bam-vcf",
        "--strict",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_pipeline_dag_validation.v1")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/configs/pipelines/local/core-germline-fastq-bam-vcf.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/core-germline-fastq-bam-vcf.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("core-germline-fastq-bam-vcf")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("cross"));
    assert_eq!(payload.get("node_count").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(payload.get("edge_count").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));

    let nodes = payload.get("nodes").and_then(serde_json::Value::as_array).expect("nodes array");
    assert!(
        nodes.iter().any(|node| {
            node.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call")
                && node.get("upstream_inputs").and_then(serde_json::Value::as_array).is_some_and(
                    |inputs| {
                        inputs.iter().any(|value| value.as_str() == Some("aligned_bam"))
                            && inputs
                                .iter()
                                .any(|value| value.as_str() == Some("coverage_report_json"))
                    },
                )
        }),
        "plan validate must keep the BAM-to-VCF call handoff explicit"
    );
}
