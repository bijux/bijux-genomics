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
fn bench_readiness_fastq_comparable_metrics_reports_governed_stage_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-fastq-comparable-metrics", "--json"]);
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.fastq_comparable_metrics.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/fastq-comparable-metrics.tsv")
    );
    assert_eq!(payload.get("comparable_stage_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("multi_tool_stage_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(
        payload.get("comparable_tool_row_count").and_then(serde_json::Value::as_u64),
        Some(12)
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("declared_stage_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(
        payload.get("missing_shared_metric_stage_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("shared_metric_field_count").and_then(serde_json::Value::as_u64),
        Some(5)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(|row| {
        row.get("comparison_contract_status").and_then(serde_json::Value::as_str)
            == Some("declared")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.index_reference")
            && row.get("tool_count").and_then(serde_json::Value::as_u64) == Some(2)
            && row.get("default_tool_id").and_then(serde_json::Value::as_str)
                == Some("bowtie2_build")
            && row.get("corpus_status").and_then(serde_json::Value::as_str) == Some("planner_only")
            && row.get("shared_metric_fields").and_then(serde_json::Value::as_array).is_some_and(
                |fields| {
                    fields == &[serde_json::Value::String("index_build_exit_code".to_string())]
                },
            )
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.validate_reads")
            && row.get("tool_count").and_then(serde_json::Value::as_u64) == Some(5)
            && row.get("default_tool_id").and_then(serde_json::Value::as_str)
                == Some("fastqvalidator")
            && row.get("corpus_status").and_then(serde_json::Value::as_str)
                == Some("fixture:corpus-01-mini")
            && row.get("shared_metric_fields").and_then(serde_json::Value::as_array).is_some_and(
                |fields| {
                    fields
                        == &[serde_json::Value::String("format_validation_pass_rate".to_string())]
                },
            )
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str)
            == Some("fastq.profile_overrepresented_sequences")
            && row.get("tool_count").and_then(serde_json::Value::as_u64) == Some(3)
            && row.get("default_tool_id").and_then(serde_json::Value::as_str) == Some("fastqc")
            && row.get("corpus_status").and_then(serde_json::Value::as_str) == Some("planner_only")
            && row.get("shared_metric_fields").and_then(serde_json::Value::as_array).is_some_and(
                |fields| {
                    fields
                        == &[
                            serde_json::Value::String("sequence_count".to_string()),
                            serde_json::Value::String("flagged_sequences".to_string()),
                            serde_json::Value::String("top_fraction".to_string()),
                        ]
                },
            )
    }));
}
