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
fn bench_readiness_all_domain_expected_result_coverage_reports_complete_active_rows() {
    let payload = run_cli_json(&[
        "bench",
        "readiness",
        "render-all-domain-expected-result-coverage",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_expected_result_coverage.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domains/expected-result-coverage.tsv")
    );
    let row_count = support::json_u64(&payload, "row_count").expect("row_count");
    assert_eq!(support::json_u64(&payload, "result_id_count"), Some(row_count));
    assert_eq!(support::json_u64(&payload, "stage_count"), Some(62));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(69));
    assert_eq!(
        payload.get("expected_result_binding_count").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );
    assert_eq!(payload.get("covered_row_count").and_then(serde_json::Value::as_u64), Some(row_count));
    assert_eq!(payload.get("missing_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("coverage_percent").and_then(serde_json::Value::as_f64), Some(100.0));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let domain_counts = support::json_object(&payload, "domain_counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(63));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(49));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(18));
    assert_eq!(support::object_u64_sum(domain_counts), row_count);

    let report_section_counts = payload
        .get("report_section_counts")
        .and_then(serde_json::Value::as_object)
        .expect("report section counts");
    assert_eq!(
        report_section_counts.get("read_cleanup").and_then(serde_json::Value::as_u64),
        Some(37)
    );
    assert_eq!(
        report_section_counts.get("variant_calling").and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(
        report_section_counts.get("normalization").and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let coverage_status_counts = support::json_object(&payload, "coverage_status_counts");
    assert_eq!(
        coverage_status_counts.get("covered").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );

    let rows = support::json_array(&payload, "rows");
    assert_eq!(rows.len() as u64, row_count);
    assert!(rows.iter().all(|row| {
        row.get("coverage_status").and_then(serde_json::Value::as_str) == Some("covered")
    }));

    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some("fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2")
            && row.get("adapter_id").and_then(serde_json::Value::as_str)
                == Some("fastq.adapter.screen_taxonomy")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("fastq.parser.screen_taxonomy")
            && row.get("schema_id").and_then(serde_json::Value::as_str)
                == Some("fastq_screen_taxonomy_v1")
            && row.get("report_section").and_then(serde_json::Value::as_str)
                == Some("contamination_screening")
    }));
    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some("bam:corpus-01-kinship-mini:bam.kinship:sample-set:king")
            && row.get("adapter_id").and_then(serde_json::Value::as_str)
                == Some("bam.adapter.kinship")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("bam.parser.kinship")
            && row.get("schema_id").and_then(serde_json::Value::as_str)
                == Some("bam_kinship_normalized_v1")
            && row.get("report_section").and_then(serde_json::Value::as_str)
                == Some("sample_identity")
    }));
    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some("vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools")
            && row.get("adapter_id").and_then(serde_json::Value::as_str)
                == Some("vcf.adapter.transform")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("vcf.parser.vcf_output")
            && row.get("schema_id").and_then(serde_json::Value::as_str)
                == Some("bijux.schemas.bench.vcf-normalized-metrics.postprocess.v1")
            && row.get("report_section").and_then(serde_json::Value::as_str)
                == Some("normalization")
    }));
    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some(
                "vcf:vcf_production_regression:vcf.imputation_metrics:vcf_cohort_with_panel:beagle",
            )
            && row.get("report_section").and_then(serde_json::Value::as_str) == Some("imputation")
            && row.get("coverage_status").and_then(serde_json::Value::as_str) == Some("covered")
    }));

    let violations =
        payload.get("violations").and_then(serde_json::Value::as_array).expect("violations");
    assert!(
        violations.is_empty(),
        "all active rows must retain complete governed expected-result coverage"
    );
}
