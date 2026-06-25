#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_core_germline_micro_pipeline_reports_real_stage_handoffs() {
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
        .args(["bench", "local", "run-core-germline-micro-pipeline", "--json"])
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
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_core_germline_micro_pipeline.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/micro/pipelines/core-germline/MICRO_PIPELINE_SUMMARY.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("core-germline-fastq-bam-vcf")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(payload.get("handoff_count").and_then(serde_json::Value::as_u64), Some(20));
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
            "bam.align",
            "bam.coverage",
            "bam.qc_pre",
            "bam.validate",
            "fastq.filter_reads",
            "fastq.profile_reads",
            "fastq.trim_reads",
            "fastq.validate_reads",
            "vcf.call",
            "vcf.filter",
            "vcf.qc",
            "vcf.stats",
        ])
    );

    let handoffs =
        payload.get("handoffs").and_then(serde_json::Value::as_array).expect("handoffs array");
    assert_eq!(handoffs.len(), 20);
    assert!(handoffs.iter().all(|handoff| {
        handoff.get("accepted").and_then(serde_json::Value::as_bool) == Some(true)
    }));
}
