#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_adna_micro_pipeline_reports_real_stage_handoffs() {
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
        .args(["bench", "local", "run-adna-micro-pipeline", "--json"])
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
        Some("bijux.bench.local_adna_micro_pipeline.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/micro/pipelines/adna/MICRO_ADNA_SUMMARY.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("adna-pseudohaploid-fastq-bam-vcf")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(payload.get("handoff_count").and_then(serde_json::Value::as_u64), Some(21));
    assert_eq!(payload.get("skipped_count").and_then(serde_json::Value::as_u64), Some(3));
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
            "bam.authenticity",
            "bam.contamination",
            "bam.coverage",
            "bam.damage",
            "bam.mapping_summary",
            "bam.validate",
            "fastq.remove_duplicates",
            "fastq.trim_terminal_damage",
            "fastq.validate_reads",
            "vcf.call_gl",
            "vcf.call_pseudohaploid",
            "vcf.damage_filter",
            "vcf.gl_propagation",
            "vcf.stats",
        ])
    );

    let skipped_stage_ids = rows
        .iter()
        .filter(|row| row.get("status").and_then(serde_json::Value::as_str) == Some("skipped"))
        .filter_map(|row| row.get("stage_id").and_then(serde_json::Value::as_str))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        skipped_stage_ids,
        BTreeSet::from(["bam.contamination", "vcf.call_gl", "vcf.gl_propagation"])
    );

    let handoffs =
        payload.get("handoffs").and_then(serde_json::Value::as_array).expect("handoffs array");
    assert_eq!(handoffs.len(), 21);
    assert!(handoffs.iter().all(|handoff| {
        handoff.get("accepted").and_then(serde_json::Value::as_bool) == Some(true)
    }));
}
