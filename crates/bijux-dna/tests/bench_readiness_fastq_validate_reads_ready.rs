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
fn bench_readiness_fastq_validate_reads_ready_reports_complete_active_validator_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-fastq-validate-reads-ready", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.fastq_validate_reads_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/fastq/validate-reads-ready.json")
    );
    assert_eq!(payload.get("active_row_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("complete_row_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("incomplete_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("checked_surface_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("sample_case_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        payload.get("validation_status_field_id").and_then(serde_json::Value::as_str),
        Some("strict_pass")
    );
    assert_eq!(
        payload.get("failure_reason_field_id").and_then(serde_json::Value::as_str),
        Some("failure_class")
    );

    let expected_tool_ids = payload
        .get("expected_tool_ids")
        .and_then(serde_json::Value::as_array)
        .expect("expected tool ids");
    assert_eq!(
        expected_tool_ids,
        &vec![
            serde_json::Value::String("fastq_scan".to_string()),
            serde_json::Value::String("fastqc".to_string()),
            serde_json::Value::String("fastqvalidator".to_string()),
            serde_json::Value::String("fqtools".to_string()),
            serde_json::Value::String("seqtk".to_string()),
        ]
    );

    let smoke_sample_ids = payload
        .get("smoke_sample_ids")
        .and_then(serde_json::Value::as_array)
        .expect("smoke sample ids");
    assert_eq!(
        smoke_sample_ids,
        &vec![
            serde_json::Value::String("toy-pe".to_string()),
            serde_json::Value::String("toy-se".to_string()),
        ]
    );
    let smoke_layouts =
        payload.get("smoke_layouts").and_then(serde_json::Value::as_array).expect("smoke layouts");
    assert_eq!(
        smoke_layouts,
        &vec![
            serde_json::Value::String("paired_end".to_string()),
            serde_json::Value::String("single_end".to_string()),
        ]
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert_eq!(rows.len(), 5);
    assert!(rows.iter().all(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.validate_reads")
            && row.get("command_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("output_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("parser_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("comparable_metrics_ready").and_then(serde_json::Value::as_bool)
                == Some(true)
            && row.get("expected_result_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("report_ready").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("smoke_parseable").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("sample_id_normalized").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("layout_normalized").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("read_count_normalized").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("validation_status_normalized").and_then(serde_json::Value::as_bool)
                == Some(true)
            && row.get("failure_reason_normalized").and_then(serde_json::Value::as_bool)
                == Some(true)
            && row.get("coverage_status").and_then(serde_json::Value::as_str) == Some("complete")
    }));

    let fastqc = rows
        .iter()
        .find(|row| row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastqc"))
        .expect("fastqc row");
    assert_eq!(
        fastqc.get("result_id").and_then(serde_json::Value::as_str),
        Some("fastq:corpus-01-mini:fastq.validate_reads:sample-set:fastqc")
    );
    assert_eq!(
        fastqc.get("report_section_id").and_then(serde_json::Value::as_str),
        Some("input_readiness")
    );
    assert_eq!(
        fastqc.get("summary_table_id").and_then(serde_json::Value::as_str),
        Some("validation_intake")
    );
    assert_eq!(
        fastqc.get("command_argv_output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/rendered-commands.argv.jsonl")
    );
    assert_eq!(
        fastqc.get("output_contract_proof_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/fastq-adapter-output-contract.tsv")
    );
    assert_eq!(
        fastqc.get("parser_proof_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/fastq-parser-coverage.tsv")
    );
    assert_eq!(
        fastqc.get("comparable_metrics_proof_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/fastq-comparable-metrics.tsv")
    );
    assert_eq!(
        fastqc.get("expected_result_proof_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/expected-benchmark-results.tsv")
    );
    assert_eq!(
        fastqc.get("smoke_proof_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/fastq.validate_reads/report.json")
    );
    assert!(fastqc.get("shared_metric_fields").and_then(serde_json::Value::as_array).is_some_and(
        |fields| {
            fields == &[serde_json::Value::String("format_validation_pass_rate".to_string())]
        }
    ));
    assert!(fastqc
        .get("validation_report_fields")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|fields| {
            fields.iter().any(|value| value.as_str() == Some("failure_class"))
                && fields.iter().any(|value| value.as_str() == Some("strict_pass"))
        }));
    assert!(fastqc
        .get("validated_reads_manifest_fields")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|fields| {
            fields.iter().any(|value| value.as_str() == Some("paired_mode"))
                && fields.iter().any(|value| value.as_str() == Some("validated_pairs"))
        }));
}
