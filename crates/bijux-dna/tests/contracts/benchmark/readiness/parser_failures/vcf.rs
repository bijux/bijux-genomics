#![allow(clippy::expect_used, clippy::too_many_lines)]

use super::{run_cli, run_cli_json};

#[test]
fn bench_readiness_vcf_parser_failure_audit_report_governed_failure_rows() {
    let payload = run_cli_json(
        "vcf-parser-failure-json-",
        &["bench", "readiness", "render-vcf-parser-failure-tests", "--json"],
    );

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_parser_failure_audit.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/vcf/vcf-parser-failure-tests.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(payload.get("passed_row_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(payload.get("failed_row_count").and_then(serde_json::Value::as_u64), Some(0));

    let counts = payload
        .get("failure_reason_counts")
        .and_then(serde_json::Value::as_object)
        .expect("failure reason counts");
    for reason in [
        "empty_output",
        "malformed_vcf",
        "missing_index",
        "missing_sample_column",
        "malformed_pca_table",
        "malformed_imputation_quality_file",
        "malformed_segment_file",
    ] {
        assert_eq!(
            counts.get(reason).and_then(serde_json::Value::as_u64),
            Some(1),
            "reason count mismatch for {reason}"
        );
    }

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 7);
    assert!(rows
        .iter()
        .all(|row| row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)));

    assert!(rows.iter().any(|row| {
        row.get("failure_reason").and_then(serde_json::Value::as_str) == Some("missing_index")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.postprocess")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("parse_bcftools_postprocess_metrics")
            && row.get("probe_artifact_path").and_then(serde_json::Value::as_str)
                == Some(
                    "artifacts/bench-readiness/vcf-parser-failure-tests/missing-index/fixture/raw.postprocess.vcf.tbi",
                )
            && row.get("observed_error").and_then(serde_json::Value::as_str).is_some_and(|value| {
                value.contains("required tabix index for postprocess output is missing")
            })
    }));

    assert!(rows.iter().any(|row| {
        row.get("failure_reason").and_then(serde_json::Value::as_str)
            == Some("missing_sample_column")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.phasing")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("shapeit5")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("parse_shapeit5_phasing_metrics")
            && row
                .get("observed_error")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value.contains("phased VCF row is missing sample columns"))
    }));

    assert!(rows.iter().any(|row| {
        row.get("failure_reason").and_then(serde_json::Value::as_str)
            == Some("malformed_imputation_quality_file")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.impute")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("parse_beagle_impute_metrics")
    }));

    assert!(rows.iter().any(|row| {
        row.get("failure_reason").and_then(serde_json::Value::as_str)
            == Some("malformed_segment_file")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.ibd")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("germline")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("parse_germline_ibd_segment_metrics")
    }));
}

#[test]
fn bench_readiness_vcf_parser_failure_audit_writes_governed_json_file() {
    let (sandbox, _output) = run_cli(
        "vcf-parser-failure-file-",
        &["bench", "readiness", "render-vcf-parser-failure-tests"],
    );
    let report_path = sandbox
        .path()
        .join("artifacts/benchmark-smoke/readiness/vcf/vcf-parser-failure-tests.json");
    assert!(report_path.is_file(), "VCF parser failure report JSON must exist");

    let payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).expect("read report"))
            .expect("parse report json");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_parser_failure_audit.v1")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(7));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert!(rows.iter().any(|row| {
        row.get("failure_reason").and_then(serde_json::Value::as_str) == Some("malformed_pca_table")
            && row.get("probe_artifact_path").and_then(serde_json::Value::as_str)
                == Some(
                    "artifacts/bench-readiness/vcf-parser-failure-tests/malformed-pca-table/fixture/raw.evec",
                )
            && row.get("expected_error_fragment").and_then(serde_json::Value::as_str)
                == Some("does not contain any numeric components")
    }));
    assert!(rows.iter().any(|row| {
        row.get("failure_reason").and_then(serde_json::Value::as_str) == Some("empty_output")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
    }));
}
