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
fn bench_readiness_bam_comparable_metrics_reports_governed_stage_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-bam-comparable-metrics", "--json"]);
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.bam_comparable_metrics.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/bam-comparable-metrics.tsv")
    );
    assert_eq!(payload.get("comparable_stage_count").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(payload.get("multi_tool_stage_count").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(
        payload.get("comparable_tool_row_count").and_then(serde_json::Value::as_u64),
        Some(40)
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(payload.get("declared_stage_count").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(
        payload.get("missing_shared_metric_stage_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("shared_metric_field_count").and_then(serde_json::Value::as_u64),
        Some(65)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 15);
    assert!(rows.iter().all(|row| {
        row.get("comparison_contract_status").and_then(serde_json::Value::as_str)
            == Some("declared")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.validate")
            && row.get("tool_count").and_then(serde_json::Value::as_u64) == Some(3)
            && row.get("default_tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
            && row.get("corpus_status").and_then(serde_json::Value::as_str)
                == Some("fixture:corpus-01-bam-mini")
            && row.get("shared_metric_fields").and_then(serde_json::Value::as_array).is_some_and(
                |fields| {
                    fields
                        == &[
                            serde_json::Value::String("validation_status".to_string()),
                            serde_json::Value::String("validation_errors".to_string()),
                            serde_json::Value::String("validation_warnings".to_string()),
                            serde_json::Value::String("input_bam_identity".to_string()),
                        ]
                },
            )
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.coverage")
            && row.get("tool_count").and_then(serde_json::Value::as_u64) == Some(3)
            && row.get("default_tool_id").and_then(serde_json::Value::as_str) == Some("mosdepth")
            && row.get("corpus_status").and_then(serde_json::Value::as_str)
                == Some("fixture:corpus-01-bam-mini")
            && row.get("shared_metric_fields").and_then(serde_json::Value::as_array).is_some_and(
                |fields| {
                    fields
                        == &[
                            serde_json::Value::String("mean_depth".to_string()),
                            serde_json::Value::String("breadth_1x".to_string()),
                            serde_json::Value::String("covered_bases".to_string()),
                            serde_json::Value::String("observed_region_count".to_string()),
                            serde_json::Value::String("region_ids".to_string()),
                        ]
                },
            )
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.damage")
            && row.get("tool_count").and_then(serde_json::Value::as_u64) == Some(6)
            && row.get("default_tool_id").and_then(serde_json::Value::as_str) == Some("mapdamage2")
            && row.get("corpus_status").and_then(serde_json::Value::as_str)
                == Some("fixture:corpus-01-adna-damage-mini")
            && row.get("shared_metric_fields").and_then(serde_json::Value::as_array).is_some_and(
                |fields| {
                    fields
                        == &[
                            serde_json::Value::String("terminal_c_to_t_5p".to_string()),
                            serde_json::Value::String("terminal_g_to_a_3p".to_string()),
                            serde_json::Value::String("damage_signal".to_string()),
                            serde_json::Value::String("runtime_s".to_string()),
                            serde_json::Value::String("memory_mb".to_string()),
                        ]
                },
            )
    }));
}
