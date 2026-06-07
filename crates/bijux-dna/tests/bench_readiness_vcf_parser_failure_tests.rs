#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json() -> serde_json::Value {
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
        .args(["bench", "readiness", "render-vcf-parser-failure-tests", "--json"])
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
fn bench_readiness_vcf_parser_failure_tests_report_governed_failure_rows() {
    let payload = run_cli_json();

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_parser_failure_tests.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf-parser-failure-tests.json")
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
