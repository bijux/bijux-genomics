#![allow(clippy::expect_used, clippy::too_many_lines)]

use super::{run_cli, run_cli_json};

#[test]
fn bench_readiness_parser_failure_audit_report_structures_parser_errors() {
    let payload = run_cli_json(
        "fastq-bam-parser-failure-json-",
        &["bench", "readiness", "render-parser-failure-tests", "--json"],
    );

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.parser_failure_audit.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/parser-failure-tests.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(99));
    assert_eq!(payload.get("passed_row_count").and_then(serde_json::Value::as_u64), Some(99));
    assert_eq!(payload.get("failed_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload
            .get("domain_row_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(48)
    );
    assert_eq!(
        payload
            .get("domain_row_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(51)
    );
    assert_eq!(
        payload
            .get("expected_failure_class_counts")
            .and_then(|value| value.get("missing_raw_output"))
            .and_then(serde_json::Value::as_u64),
        Some(33)
    );
    assert_eq!(
        payload
            .get("expected_failure_class_counts")
            .and_then(|value| value.get("empty_raw_output"))
            .and_then(serde_json::Value::as_u64),
        Some(33)
    );
    assert_eq!(
        payload
            .get("expected_failure_class_counts")
            .and_then(|value| value.get("malformed_raw_output"))
            .and_then(serde_json::Value::as_u64),
        Some(33)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 99);
    assert!(rows.iter().all(|row| {
        row.get("passed") == Some(&serde_json::Value::Bool(true))
            && row
                .get("observed_error")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
    }));

    let fastq_row = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.detect_adapters")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastqc")
                && row.get("expected_failure_class").and_then(serde_json::Value::as_str)
                    == Some("malformed_raw_output")
        })
        .expect("fastq malformed parser row");
    assert!(fastq_row
        .get("observed_error")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| value.contains("fastqc total sequences missing")));

    let bam_row = rows
        .iter()
        .find(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.qc_pre")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
                && row.get("expected_failure_class").and_then(serde_json::Value::as_str)
                    == Some("malformed_raw_output")
        })
        .expect("bam malformed parser row");
    assert!(bam_row
        .get("observed_error")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| value.contains("flagstat summary missing `in total` line")));
    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
            && row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.filter_low_complexity")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("dustmasker")
            && row.get("expected_failure_class").and_then(serde_json::Value::as_str)
                == Some("missing_raw_output")
    }));
}

#[test]
fn bench_readiness_parser_failure_audit_file_writes_self_describing_report() {
    let (sandbox, output) = run_cli(
        "fastq-bam-parser-failure-file-",
        &["bench", "readiness", "render-parser-failure-tests"],
    );
    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(
        rendered_path.trim(),
        "artifacts/benchmark-smoke/readiness/parser-failure-tests.json"
    );

    let payload = std::fs::read_to_string(sandbox.path().join(rendered_path.trim()))
        .expect("read parser failure tests report");
    let report: serde_json::Value = serde_json::from_str(&payload).expect("parse report JSON");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.parser_failure_audit.v1")
    );
    assert_eq!(report.get("row_count").and_then(serde_json::Value::as_u64), Some(99));
    assert_eq!(report.get("passed_row_count").and_then(serde_json::Value::as_u64), Some(99));
    assert_eq!(report.get("failed_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(report.get("rows").and_then(serde_json::Value::as_array).map(Vec::len), Some(99));
}
