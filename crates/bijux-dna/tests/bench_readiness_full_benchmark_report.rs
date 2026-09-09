#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let sandbox = support::RepoSandbox::new("bench-readiness-full-benchmark-report")
        .expect("isolated report repository");
    let home = tempfile::tempdir().expect("tempdir");

    let output = sandbox
        .bijux_dna_command()
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

    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse stdout as json");
    let json_path = payload["json_output_path"].as_str().expect("JSON report path");
    let persisted: serde_json::Value = serde_json::from_slice(
        &std::fs::read(sandbox.path().join(json_path)).expect("read persisted full report"),
    )
    .expect("parse persisted full report");
    assert_eq!(persisted, payload, "persisted full report must match stdout");
    let markdown_path = payload["markdown_output_path"].as_str().expect("Markdown report path");
    let markdown =
        std::fs::read_to_string(sandbox.path().join(markdown_path)).expect("read markdown");
    assert_markdown_sections(&markdown);
    payload
}

#[test]
fn bench_readiness_full_benchmark_report_tracks_governed_report_sections() -> anyhow::Result<()> {
    let payload = run_cli_json(&["bench", "readiness", "render-full-benchmark-report", "--json"]);
    assert_report_owner_contract(&payload)?;

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.full_benchmark_report.v1")
    );
    assert_eq!(
        payload.get("markdown_output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/all-domains/FASTQ_BAM_VCF_BENCHMARK_REPORT.md")
    );
    assert_eq!(
        payload.get("json_output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/all-domains/FASTQ_BAM_VCF_BENCHMARK_REPORT.json")
    );
    let row_count = support::json_u64(&payload, "row_count").expect("row_count");
    let expected_result_row_count = support::json_u64(&payload, "expected_result_row_count")
        .expect("expected_result_row_count");
    assert_eq!(
        payload.get("explicit_unsupported_row_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    let present_row_count =
        support::json_u64(&payload, "present_row_count").expect("present_row_count");
    let missing_result_row_count =
        support::json_u64(&payload, "missing_result_row_count").expect("missing_result_row_count");
    let unsupported_pair_row_count = support::json_u64(&payload, "unsupported_pair_row_count")
        .expect("unsupported_pair_row_count");
    assert_eq!(missing_result_row_count, 3);
    assert_eq!(unsupported_pair_row_count, 1);
    assert_eq!(present_row_count + missing_result_row_count, expected_result_row_count);
    assert_eq!(row_count, expected_result_row_count + unsupported_pair_row_count);
    assert_eq!(
        payload.get("failure_row_count").and_then(serde_json::Value::as_u64),
        Some(expected_result_row_count)
    );
    assert_eq!(payload.get("failure_class_row_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(
        payload.get("passes_behavior_test").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let status_counts = payload
        .get("row_status_counts")
        .and_then(serde_json::Value::as_object)
        .expect("row status counts");
    assert_eq!(
        status_counts.get("present").and_then(serde_json::Value::as_u64),
        Some(present_row_count)
    );
    assert_eq!(
        status_counts.get("missing_result").and_then(serde_json::Value::as_u64),
        Some(missing_result_row_count)
    );
    assert_eq!(
        status_counts.get("unsupported_pair").and_then(serde_json::Value::as_u64),
        Some(unsupported_pair_row_count)
    );

    let rows = support::json_array(&payload, "rows")?;
    assert_eq!(rows.len() as u64, row_count);

    let result_ids = rows
        .iter()
        .filter_map(|row| row.get("result_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(result_ids.len() as u64, expected_result_row_count);

    let missing_result_ids = rows
        .iter()
        .filter(|row| {
            row.get("row_status").and_then(serde_json::Value::as_str) == Some("missing_result")
        })
        .filter_map(|row| row.get("result_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(missing_result_ids.len(), 3);
    assert!(missing_result_ids
        .contains("fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2"));
    assert!(missing_result_ids.contains("bam:corpus-01-bam-mini:bam.coverage:sample-set:samtools"));
    assert!(
        missing_result_ids.contains("vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools")
    );

    let unsupported_row = rows
        .iter()
        .find(|row| {
            row.get("row_status").and_then(serde_json::Value::as_str) == Some("unsupported_pair")
        })
        .expect("unsupported row");
    assert_eq!(
        unsupported_row.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.filter")
    );
    assert_eq!(
        unsupported_row.get("tool_id").and_then(serde_json::Value::as_str),
        Some("samtools")
    );
    assert_eq!(
        unsupported_row.get("report_section").and_then(serde_json::Value::as_str),
        Some("unsupported_pairs")
    );

    let runtime = support::json_array(&payload, "runtime")?;
    assert_eq!(runtime.len() as u64, row_count);
    let memory = support::json_array(&payload, "memory")?;
    assert_eq!(memory.len() as u64, row_count);

    let failures = payload.get("failures").expect("failures section");
    assert_eq!(
        failures.get("simulated_failure_row_count").and_then(serde_json::Value::as_u64),
        Some(expected_result_row_count)
    );
    assert_eq!(
        failures.get("failure_class_row_count").and_then(serde_json::Value::as_u64),
        Some(8)
    );

    let missing_results = payload
        .get("missing_results")
        .and_then(serde_json::Value::as_array)
        .expect("missing results");
    assert_eq!(missing_results.len(), 3);

    let unsupported_pairs = payload
        .get("unsupported_pairs")
        .and_then(serde_json::Value::as_array)
        .expect("unsupported pairs");
    assert_eq!(unsupported_pairs.len(), 1);
    Ok(())
}

fn assert_report_owner_contract(payload: &serde_json::Value) -> anyhow::Result<()> {
    use bijux_dna_api::v1::api::bench::{
        validate_full_benchmark_report, FullBenchmarkReport, FullBenchmarkReportAdmissionContext,
    };
    let report: FullBenchmarkReport = serde_json::from_value(payload.clone())?;
    assert_eq!(
        report.present_row_count + report.missing_result_row_count,
        report.expected_result_row_count
    );
    assert_eq!(
        report.row_count,
        report.expected_result_row_count + report.unsupported_pair_row_count
    );
    assert_eq!(report.explicit_unsupported_row_count, report.unsupported_pairs.len());
    assert_eq!(report.missing_result_row_count, report.missing_results.len());
    assert_eq!(report.failure_row_count, report.expected_result_row_count);
    assert_eq!(report.failure_class_row_count, report.failures.classification_rows.len());
    assert_eq!(report.rows.len(), report.row_count);
    assert_eq!(report.runtime.len(), report.row_count);
    assert_eq!(report.memory.len(), report.row_count);
    assert!(!report.rows.is_empty(), "full benchmark report should contain benchmark rows");
    assert!(report.comparable_metrics.iter().all(|row| !row.direction.is_empty()));
    assert!(report.comparable_metrics.iter().all(|row| !row.tolerance_kind.is_empty()));
    assert!(report.comparable_metrics.iter().all(|row| !row.pass_rule.is_empty()));
    assert!(report.comparable_metrics.iter().all(|row| !row.insufficiency_behavior.is_empty()));
    assert!(report.passes_behavior_test);

    let context = FullBenchmarkReportAdmissionContext {
        expected_result_ids: report.rows.iter().filter_map(|row| row.result_id.clone()).collect(),
        collector_missing_result_count: report.missing_result_row_count,
        collector_failure_count: report.failure_row_count,
        collector_unsupported_pair_count: report.explicit_unsupported_row_count,
        triggered_failure_class_count: report.failure_class_row_count,
    };
    let mut unsafe_report = report.clone();
    unsafe_report.rows[0].evidence_path = "../outside.json".to_string();
    assert!(validate_full_benchmark_report(&mut unsafe_report, &context).is_err());
    Ok(())
}

fn assert_markdown_sections(markdown: &str) {
    assert!(markdown.contains("# FASTQ + BAM + VCF Benchmark Report"));
    assert!(markdown.contains("## Stage-Centric"));
    assert!(markdown.contains("## Tool-Centric"));
    assert!(markdown.contains("## Corpus-Centric"));
    assert!(markdown.contains("## Pipeline-Centric"));
    assert!(markdown.contains("## Runtime"));
    assert!(markdown.contains("## Memory"));
    assert!(markdown.contains("## Failures"));
    assert!(markdown.contains("## Missing Results"));
    assert!(markdown.contains("## Comparable Metrics"));
    assert!(markdown.contains("## Unsupported Pairs"));
    assert!(markdown.contains("Tolerance Kind"));
    assert!(markdown.contains("Insufficiency Behavior"));
    assert!(markdown.contains("match_reference_structure"));
    assert!(markdown.contains("fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2"));
    assert!(markdown.contains("vcf.filter"));
    assert!(markdown.contains("samtools"));
}
