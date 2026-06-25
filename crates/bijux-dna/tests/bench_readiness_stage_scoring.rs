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
fn bench_readiness_stage_scoring_reports_governed_stage_decision_contracts() {
    let payload = run_cli_json(&["bench", "readiness", "render-stage-scoring", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.stage_scoring.v1")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/bench/local/stage-scoring.toml")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(69));
    assert_eq!(payload.get("multi_tool_stage_count").and_then(serde_json::Value::as_u64), Some(31));
    assert_eq!(
        payload.get("single_tool_stage_count").and_then(serde_json::Value::as_u64),
        Some(38)
    );
    assert_eq!(payload.get("scientific_stage_count").and_then(serde_json::Value::as_u64), Some(29));
    assert_eq!(payload.get("failure_class_count").and_then(serde_json::Value::as_u64), Some(7));

    let domain_counts =
        payload.get("domain_counts").and_then(serde_json::Value::as_object).expect("domain counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(27));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(24));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(18));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 69);

    let validate_reads = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.validate_reads")
        })
        .expect("fastq.validate_reads row");
    assert_eq!(
        validate_reads.get("decision_mode").and_then(serde_json::Value::as_str),
        Some("multi_tool_ranking")
    );
    assert_eq!(
        validate_reads.get("default_tool_id").and_then(serde_json::Value::as_str),
        Some("fastqvalidator")
    );
    assert_eq!(
        validate_reads.get("benchmark_ready_tool_count").and_then(serde_json::Value::as_u64),
        Some(5)
    );
    assert_eq!(
        validate_reads.get("recommendation_gate").and_then(serde_json::Value::as_str),
        Some("rank benchmark-ready tools only after completion and failure-class gates pass")
    );
    assert_eq!(
        validate_reads
            .get("correctness")
            .and_then(|value| value.get("signal"))
            .and_then(serde_json::Value::as_str),
        Some("scientific_comparable_metrics")
    );
    assert_eq!(
        validate_reads
            .get("scientific_threshold")
            .and_then(|value| value.get("applicability"))
            .and_then(serde_json::Value::as_str),
        Some("required")
    );
    assert_eq!(
        validate_reads
            .get("scientific_threshold")
            .and_then(|value| value.get("metric_ids"))
            .and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::Value::String("format_validation_pass_rate".to_string(),)])
    );
    assert_eq!(
        validate_reads
            .get("failure_class")
            .and_then(|value| value.get("blocking_class_ids"))
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(6)
    );

    let correct_errors = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.correct_errors")
        })
        .expect("fastq.correct_errors row");
    assert_eq!(
        correct_errors.get("decision_mode").and_then(serde_json::Value::as_str),
        Some("multi_tool_ranking")
    );
    assert_eq!(
        correct_errors
            .get("correctness")
            .and_then(|value| value.get("signal"))
            .and_then(serde_json::Value::as_str),
        Some("output_contract")
    );
    assert_eq!(
        correct_errors
            .get("scientific_threshold")
            .and_then(|value| value.get("applicability"))
            .and_then(serde_json::Value::as_str),
        Some("not_applicable")
    );

    let complexity = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.complexity")
        })
        .expect("bam.complexity row");
    assert_eq!(
        complexity.get("decision_mode").and_then(serde_json::Value::as_str),
        Some("single_tool_acceptance")
    );
    assert_eq!(
        complexity.get("default_tool_id").and_then(serde_json::Value::as_str),
        Some("preseq")
    );
    assert_eq!(
        complexity.get("benchmark_ready_tool_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        complexity
            .get("correctness")
            .and_then(|value| value.get("signal"))
            .and_then(serde_json::Value::as_str),
        Some("output_contract")
    );
    assert_eq!(
        complexity
            .get("scientific_threshold")
            .and_then(|value| value.get("applicability"))
            .and_then(serde_json::Value::as_str),
        Some("not_applicable")
    );
    assert_eq!(
        complexity
            .get("weights")
            .and_then(|value| value.get("scientific_threshold"))
            .and_then(serde_json::Value::as_f64),
        Some(0.0)
    );
}

#[test]
fn bench_readiness_stage_scoring_validation_reports_governed_config_contract() {
    let render_output = run_cli(&["bench", "readiness", "render-stage-scoring"]);
    assert!(
        render_output.status.success(),
        "render command failed: {}\nstdout:\n{}\nstderr:\n{}",
        render_output.status,
        String::from_utf8_lossy(&render_output.stdout),
        String::from_utf8_lossy(&render_output.stderr)
    );

    let payload = run_cli_json(&["bench", "readiness", "validate-stage-scoring", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.stage_scoring.v1")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/bench/local/stage-scoring.toml")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(69));
    assert_eq!(payload.get("scientific_stage_count").and_then(serde_json::Value::as_u64), Some(29));
}
