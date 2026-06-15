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
fn bench_readiness_all_domain_parser_fixture_coverage_reports_complete_active_rows() {
    let payload = run_cli_json(&[
        "bench",
        "readiness",
        "render-all-domain-parser-fixture-coverage",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_parser_fixture_coverage.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/all-domains/parser-fixture-coverage.tsv")
    );
    let row_count = support::json_u64(&payload, "row_count").expect("row_count");
    assert!(payload
        .get("stage_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count >= 62));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(71));
    assert_eq!(
        payload.get("parser_proof_binding_count").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );
    assert_eq!(
        payload.get("covered_row_count").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );
    assert_eq!(payload.get("missing_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("coverage_percent").and_then(serde_json::Value::as_f64), Some(100.0));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let domain_counts = support::json_object(&payload, "domain_counts");
    assert_eq!(support::object_u64(domain_counts, "fastq"), Some(69));
    assert_eq!(support::object_u64(domain_counts, "bam"), Some(49));
    assert_eq!(support::object_u64(domain_counts, "vcf"), Some(20));
    assert_eq!(support::object_u64_sum(domain_counts), row_count);

    let proof_source_counts = payload
        .get("proof_source_counts")
        .and_then(serde_json::Value::as_object)
        .expect("proof source counts");
    assert_eq!(
        proof_source_counts
            .get("fastq_parser_fixture_coverage")
            .and_then(serde_json::Value::as_u64),
        Some(69)
    );
    assert_eq!(
        proof_source_counts.get("bam_parser_coverage").and_then(serde_json::Value::as_u64),
        Some(49)
    );
    assert_eq!(
        proof_source_counts.get("vcf_parser_fixture_coverage").and_then(serde_json::Value::as_u64),
        Some(20)
    );
    assert_eq!(
        proof_source_counts.values().filter_map(serde_json::Value::as_u64).sum::<u64>(),
        row_count
    );

    let coverage_status_counts = support::json_object(&payload, "coverage_status_counts");
    assert_eq!(
        coverage_status_counts.get("covered").and_then(serde_json::Value::as_u64),
        Some(row_count)
    );
    assert_eq!(coverage_status_counts.len(), 1);

    let rows = support::json_array(&payload, "rows");
    assert_eq!(rows.len() as u64, row_count);
    assert!(rows.iter().all(|row| {
        row.get("coverage_status").and_then(serde_json::Value::as_str) == Some("covered")
    }));

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.trim_reads")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("trimmomatic")
            && row.get("parser_fixture_reference_kind").and_then(serde_json::Value::as_str)
                == Some("fixture_case")
            && row.get("parser_fixture_reference").and_then(serde_json::Value::as_str)
                == Some("fastq.trim_reads.report_json")
            && row.get("proof_source").and_then(serde_json::Value::as_str)
                == Some("fastq_parser_fixture_coverage")
    }));
    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.contamination")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("schmutzi")
            && row.get("parser_fixture_reference_kind").and_then(serde_json::Value::as_str)
                == Some("fixture_corpus")
            && row.get("parser_fixture_reference").and_then(serde_json::Value::as_str)
                == Some("fixture:corpus-01-adna-bam-mini")
            && row.get("proof_source").and_then(serde_json::Value::as_str)
                == Some("bam_parser_coverage")
    }));
    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.postprocess")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("parser_fixture_parser_id").and_then(serde_json::Value::as_str)
                == Some("parse_bcftools_postprocess_metrics")
            && row.get("parser_fixture_schema_id").and_then(serde_json::Value::as_str)
                == Some("bijux.vcf.postprocess.v1")
            && row.get("parser_fixture_reference_kind").and_then(serde_json::Value::as_str)
                == Some("fixture_directory")
            && row.get("parser_fixture_reference").and_then(serde_json::Value::as_str).is_some_and(
                |value| {
                    value.starts_with(
                        "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.postprocess",
                    )
                },
            )
            && row.get("proof_source").and_then(serde_json::Value::as_str)
                == Some("vcf_parser_fixture_coverage")
    }));
    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
            && row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.index_reference")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2_build")
            && row.get("parser_fixture_reference_kind").and_then(serde_json::Value::as_str)
                == Some("fixture_case")
            && row.get("parser_fixture_reference").and_then(serde_json::Value::as_str)
                == Some("fastq.index_reference.report_json")
            && row.get("parser_fixture_parser_id").and_then(serde_json::Value::as_str)
                == Some("parse_index_reference_report")
            && row.get("proof_source").and_then(serde_json::Value::as_str)
                == Some("fastq_parser_fixture_coverage")
    }));

    let violations =
        payload.get("violations").and_then(serde_json::Value::as_array).expect("violations");
    assert!(violations.is_empty(), "all active rows must retain parser proof");
}
