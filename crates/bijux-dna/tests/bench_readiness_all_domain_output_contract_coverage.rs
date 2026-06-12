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
fn bench_readiness_all_domain_output_contract_coverage_reports_complete_active_rows() {
    let payload = run_cli_json(&[
        "bench",
        "readiness",
        "render-all-domain-output-contract-coverage",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_output_contract_coverage.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domains/output-contract-coverage.tsv")
    );
    let row_count = support::json_u64(&payload, "row_count").expect("row_count");
    assert_eq!(payload.get("result_id_count").and_then(serde_json::Value::as_u64), Some(row_count));
    assert!(payload
        .get("stage_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count >= 62));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(69));
    assert_eq!(
        payload.get("output_declaration_binding_count").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );
    assert_eq!(
        payload.get("source_proof_binding_count").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );
    assert_eq!(
        payload.get("covered_row_count").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );
    assert_eq!(payload.get("missing_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("coverage_percent").and_then(serde_json::Value::as_f64), Some(100.0));
    assert_eq!(
        payload.get("raw_output_declared_row_count").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );
    assert_eq!(
        payload.get("normalized_metrics_declared_row_count").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );
    assert_eq!(
        payload.get("logs_declared_row_count").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );
    assert_eq!(
        payload.get("manifest_declared_row_count").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );
    assert_eq!(
        payload.get("index_required_row_count").and_then(serde_json::Value::as_u64),
        payload.get("index_declared_row_count").and_then(serde_json::Value::as_u64)
    );
    let index_declared_row_count =
        support::json_u64(&payload, "index_declared_row_count").expect("index_declared_row_count");
    let index_not_applicable_row_count =
        support::json_u64(&payload, "index_not_applicable_row_count")
            .expect("index_not_applicable_row_count");
    assert!(index_declared_row_count > 0);
    assert!(index_declared_row_count + index_not_applicable_row_count <= row_count);
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let domain_counts = support::json_object(&payload, "domain_counts");
    assert_eq!(support::object_u64(domain_counts, "fastq"), Some(63));
    assert_eq!(support::object_u64(domain_counts, "bam"), Some(49));
    assert_eq!(support::object_u64(domain_counts, "vcf"), Some(20));
    assert_eq!(support::object_u64_sum(domain_counts), row_count);

    let proof_source_counts = payload
        .get("proof_source_counts")
        .and_then(serde_json::Value::as_object)
        .expect("proof source counts");
    assert_eq!(
        proof_source_counts.get("fastq_output_contract").and_then(serde_json::Value::as_u64),
        Some(63)
    );
    assert_eq!(
        proof_source_counts.get("bam_output_contract").and_then(serde_json::Value::as_u64),
        Some(49)
    );
    assert_eq!(
        proof_source_counts.get("vcf_output_contract").and_then(serde_json::Value::as_u64),
        Some(20)
    );
    assert_eq!(
        proof_source_counts.values().filter_map(serde_json::Value::as_u64).sum::<u64>(),
        row_count
    );

    let index_coverage_counts = payload
        .get("index_coverage_counts")
        .and_then(serde_json::Value::as_object)
        .expect("index coverage counts");
    assert_eq!(
        index_coverage_counts.get("covered").and_then(serde_json::Value::as_u64),
        Some(index_declared_row_count)
    );
    assert_eq!(
        index_coverage_counts.get("not_applicable").and_then(serde_json::Value::as_u64),
        Some(index_not_applicable_row_count)
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
            && row.get("proof_source").and_then(serde_json::Value::as_str)
                == Some("fastq_output_contract")
            && row.get("source_contract_status").and_then(serde_json::Value::as_str)
                == Some("complete")
            && row.get("output_declaration_status").and_then(serde_json::Value::as_str)
                == Some("complete")
            && row.get("raw_output_ids").and_then(serde_json::Value::as_array).is_some_and(
                |items| items.iter().any(|value| value.as_str() == Some("screen_report_tsv")),
            )
            && row.get("normalized_metric_ids").and_then(serde_json::Value::as_array).is_some_and(
                |items| {
                    items.iter().any(|value| value.as_str() == Some("classification_report_json"))
                },
            )
            && row.get("index_coverage_status").and_then(serde_json::Value::as_str)
                == Some("not_applicable")
    }));
    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some("bam:corpus-01-kinship-mini:bam.kinship:sample-set:king")
            && row.get("proof_source").and_then(serde_json::Value::as_str)
                == Some("bam_output_contract")
            && row.get("source_contract_status").and_then(serde_json::Value::as_str)
                == Some("complete")
            && row.get("output_declaration_status").and_then(serde_json::Value::as_str)
                == Some("complete")
            && row.get("normalized_metric_ids").and_then(serde_json::Value::as_array).is_some_and(
                |items| items.iter().any(|value| value.as_str() == Some("kinship_report")),
            )
            && row.get("index_coverage_status").and_then(serde_json::Value::as_str)
                == Some("not_applicable")
    }));
    assert!(rows.iter().any(|row| {
        row.get("result_id").and_then(serde_json::Value::as_str)
            == Some("vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools")
            && row.get("proof_source").and_then(serde_json::Value::as_str)
                == Some("vcf_output_contract")
            && row.get("source_contract_status").and_then(serde_json::Value::as_str)
                == Some("complete")
            && row.get("output_declaration_status").and_then(serde_json::Value::as_str)
                == Some("complete")
            && row.get("index_output_ids").and_then(serde_json::Value::as_array).is_some_and(
                |items| items.iter().any(|value| value.as_str() == Some("called_vcf_tbi")),
            )
            && row.get("index_coverage_status").and_then(serde_json::Value::as_str)
                == Some("covered")
    }));

    let violations =
        payload.get("violations").and_then(serde_json::Value::as_array).expect("violations");
    assert!(
        violations.is_empty(),
        "all active rows must retain complete governed output contracts"
    );
}
