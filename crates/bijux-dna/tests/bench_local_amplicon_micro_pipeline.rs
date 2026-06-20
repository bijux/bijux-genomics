#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_amplicon_micro_pipeline_reports_validated_amplicon_outputs() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _repo_lock =
        support::RepoProcessLock::acquire("micro-benchmark-mutators").expect("repo lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "local", "run-amplicon-micro-pipeline"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(
        rendered_path.trim(),
        "runs/bench/micro/pipelines/amplicon/MICRO_AMPLICON_SUMMARY.json"
    );
    let payload: serde_json::Value = serde_json::from_slice(
        &std::fs::read(repo_root.join(rendered_path.trim())).expect("read summary"),
    )
    .expect("parse summary as json");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_amplicon_micro_pipeline.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/micro/pipelines/amplicon/MICRO_AMPLICON_SUMMARY.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("amplicon-asv-otu-no-vcf")
    );
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("handoff_count").and_then(serde_json::Value::as_u64), Some(13));
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
            "benchmark.amplicon_corpus_fixture",
            "benchmark.amplicon_output_judgment",
            "benchmark.amplicon_truth_fixture",
            "fastq.cluster_otus",
            "fastq.infer_asvs",
            "fastq.normalize_abundance",
            "fastq.normalize_primers",
            "fastq.remove_chimeras",
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
        row.get("stage_id").and_then(serde_json::Value::as_str)
            == Some("benchmark.amplicon_output_judgment")
            && row
                .get("metrics")
                .and_then(serde_json::Value::as_object)
                .and_then(|metrics| metrics.get("valid"))
                .and_then(serde_json::Value::as_bool)
                == Some(true)
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.normalize_primers")
            && row
                .get("metrics")
                .and_then(serde_json::Value::as_object)
                .and_then(|metrics| metrics.get("matched_reads"))
                .and_then(serde_json::Value::as_u64)
                == Some(2)
    }));

    let handoffs =
        payload.get("handoffs").and_then(serde_json::Value::as_array).expect("handoffs array");
    assert_eq!(handoffs.len(), 13);
    assert!(handoffs.iter().all(|handoff| {
        handoff.get("accepted").and_then(serde_json::Value::as_bool) == Some(true)
    }));
}
