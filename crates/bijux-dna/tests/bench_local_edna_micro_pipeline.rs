#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_edna_micro_pipeline_reports_taxonomy_handoffs() {
    let sandbox =
        support::RepoSandbox::new("bench-local-edna-micro-pipeline-").expect("isolated repository");
    let home = tempfile::tempdir().expect("tempdir");

    let output = sandbox
        .bijux_dna_command()
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "local", "run-edna-micro-pipeline", "--json"])
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
    let report_path = payload["output_path"].as_str().expect("reported output path");
    let written: serde_json::Value = serde_json::from_slice(
        &std::fs::read(sandbox.path().join(report_path)).expect("read private workspace report"),
    )
    .expect("parse private workspace report");
    assert_eq!(written, payload, "stdout must match the report in this test's private workspace");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_edna_micro_pipeline.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/micro/pipelines/edna/MICRO_EDNA_SUMMARY.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("edna-taxonomy-fastq")
    );
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("handoff_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(
        payload.get("passes_behavior_test").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    let stage_ids = rows
        .iter()
        .filter_map(|row| row.get("stage_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        stage_ids,
        BTreeSet::from([
            "benchmark.edna_corpus_fixture",
            "benchmark.taxonomy_database_fixture",
            "benchmark.taxonomy_output_judgment",
            "fastq.screen_taxonomy",
            "fastq.validate_reads",
        ])
    );
    assert!(rows
        .iter()
        .all(|row| { row.get("status").and_then(serde_json::Value::as_str) == Some("succeeded") }));
    assert!(rows.iter().all(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) != Some("bam")
            && row.get("domain").and_then(serde_json::Value::as_str) != Some("vcf")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.screen_taxonomy")
            && row
                .get("metrics")
                .and_then(serde_json::Value::as_object)
                .and_then(|metrics| metrics.get("unclassified_output_count"))
                .and_then(serde_json::Value::as_u64)
                == Some(2)
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str)
            == Some("benchmark.taxonomy_output_judgment")
            && row
                .get("metrics")
                .and_then(serde_json::Value::as_object)
                .and_then(|metrics| metrics.get("false_positive_count"))
                .and_then(serde_json::Value::as_u64)
                == Some(0)
    }));

    let handoffs =
        payload.get("handoffs").and_then(serde_json::Value::as_array).expect("handoffs array");
    assert_eq!(handoffs.len(), 8);
    assert!(handoffs.iter().all(|handoff| {
        handoff.get("accepted").and_then(serde_json::Value::as_bool) == Some(true)
    }));
}
