#![allow(clippy::expect_used)]

use std::collections::BTreeSet;
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
fn bench_readiness_all_domain_parser_collector_reports_fake_and_real_smoke_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-all-domain-parser-collector", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_parser_collector.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/parser-collector-all-domains.json")
    );
    assert_eq!(
        payload.get("fixture_root").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/parser-collector-all-domains-fixture")
    );
    assert_eq!(
        payload.get("fake_run_root").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/parser-collector-all-domains-fixture/fake-runs")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(123));
    assert_eq!(payload.get("fake_run_row_count").and_then(serde_json::Value::as_u64), Some(120));
    assert_eq!(payload.get("real_smoke_row_count").and_then(serde_json::Value::as_u64), Some(3));

    let source_kind_counts = payload
        .get("source_kind_counts")
        .and_then(serde_json::Value::as_object)
        .expect("source kind counts");
    assert_eq!(source_kind_counts.get("fake_run").and_then(serde_json::Value::as_u64), Some(120));
    assert_eq!(source_kind_counts.get("real_smoke").and_then(serde_json::Value::as_u64), Some(3));

    let domain_counts =
        payload.get("domain_counts").and_then(serde_json::Value::as_object).expect("domain counts");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(64));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(50));
    assert_eq!(domain_counts.get("vcf").and_then(serde_json::Value::as_u64), Some(9));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 123);

    let fake_result_ids = rows
        .iter()
        .filter(|row| {
            row.get("source_kind").and_then(serde_json::Value::as_str) == Some("fake_run")
        })
        .filter_map(|row| row.get("result_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(fake_result_ids.len(), 120);

    let fastq_smoke = rows
        .iter()
        .find(|row| {
            row.get("record_id").and_then(serde_json::Value::as_str)
                == Some("real-smoke:fastq.validate_reads")
        })
        .expect("fastq smoke row");
    assert_eq!(
        fastq_smoke.get("document_kind").and_then(serde_json::Value::as_str),
        Some("fastq_local_smoke_report")
    );
    assert_eq!(
        fastq_smoke.get("parsed_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.fastq.validate.local_smoke.report.v1")
    );
    assert_eq!(
        fastq_smoke
            .get("normalized_snapshot")
            .and_then(|value| value.get("case_count"))
            .and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        fastq_smoke
            .get("normalized_snapshot")
            .and_then(|value| value.get("all_cases_passed"))
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let bam_smoke = rows
        .iter()
        .find(|row| {
            row.get("record_id").and_then(serde_json::Value::as_str)
                == Some("real-smoke:bam.validate")
        })
        .expect("bam smoke row");
    assert_eq!(
        bam_smoke.get("document_kind").and_then(serde_json::Value::as_str),
        Some("bam_local_smoke_report")
    );
    assert_eq!(
        bam_smoke.get("parsed_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bam.validate.local_smoke.report.v1")
    );
    assert_eq!(
        bam_smoke
            .get("normalized_snapshot")
            .and_then(|value| value.get("pass_case_count"))
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        bam_smoke
            .get("normalized_snapshot")
            .and_then(|value| value.get("refusal_case_count"))
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let vcf_smoke = rows
        .iter()
        .find(|row| {
            row.get("record_id").and_then(serde_json::Value::as_str) == Some("real-smoke:vcf.stats")
        })
        .expect("vcf smoke row");
    assert_eq!(
        vcf_smoke.get("document_kind").and_then(serde_json::Value::as_str),
        Some("vcf_local_smoke_metrics")
    );
    assert_eq!(
        vcf_smoke.get("parsed_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_stats_smoke.metrics.v1")
    );
    assert_eq!(
        vcf_smoke
            .get("normalized_snapshot")
            .and_then(|value| value.get("variant_count"))
            .and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(
        vcf_smoke.get("manifest_status").and_then(serde_json::Value::as_str),
        Some("succeeded")
    );
    assert_eq!(vcf_smoke.get("manifest_exit_code").and_then(serde_json::Value::as_i64), Some(0));

    let fake_vcf = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools")
        })
        .expect("fake vcf row");
    assert_eq!(
        fake_vcf.get("document_kind").and_then(serde_json::Value::as_str),
        Some("all_domain_fake_run_metrics")
    );
    assert_eq!(
        fake_vcf.get("parsed_schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_all_domain_fake_run_metrics.v1")
    );
    assert_eq!(
        fake_vcf
            .get("normalized_snapshot")
            .and_then(|value| value.get("declared_output_count"))
            .and_then(serde_json::Value::as_u64),
        Some(2)
    );
}
