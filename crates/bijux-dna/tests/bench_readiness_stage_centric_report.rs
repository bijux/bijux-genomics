#![cfg(feature = "bam_downstream")]
#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let output = run_cli(args);
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
fn bench_readiness_stage_centric_report_tracks_multi_tool_stage_coverage() {
    let payload = run_cli_json(&["bench", "readiness", "render-stage-centric-report", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.stage_centric_report.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/stage-centric-report.md")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(payload.get("multi_tool_stage_count").and_then(serde_json::Value::as_u64), Some(30));
    assert_eq!(payload.get("blocked_stage_count").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(
        payload.get("declared_shared_metric_stage_count").and_then(serde_json::Value::as_u64),
        Some(18)
    );
    assert_eq!(
        payload.get("not_declared_shared_metric_stage_count").and_then(serde_json::Value::as_u64),
        Some(12)
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(123));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(112)
    );
    assert_eq!(payload.get("blocked_row_count").and_then(serde_json::Value::as_u64), Some(11));
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(24)
    );
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(27)
    );

    let stages = payload.get("stages").and_then(serde_json::Value::as_array).expect("stages array");
    assert_eq!(stages.len(), 51);

    let trim_reads = stages
        .iter()
        .find(|stage| {
            stage.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && stage.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.trim_reads")
        })
        .expect("trim reads stage");
    assert_eq!(trim_reads.get("tool_count").and_then(serde_json::Value::as_u64), Some(14));
    assert_eq!(trim_reads.get("blocked_tool_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        trim_reads.get("comparison_contract_status").and_then(serde_json::Value::as_str),
        Some("not_declared")
    );
    assert_eq!(
        trim_reads
            .get("blocked_tool_ids")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["seqpurge (support)"])
    );

    let index_reference = stages
        .iter()
        .find(|stage| {
            stage.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && stage.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.index_reference")
        })
        .expect("index reference stage");
    assert_eq!(
        index_reference.get("blocked_tool_count").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        index_reference.get("comparison_contract_status").and_then(serde_json::Value::as_str),
        Some("declared")
    );
    assert_eq!(
        index_reference
            .get("shared_metric_fields")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["index_build_exit_code"])
    );

    let profile_overrepresented = stages
        .iter()
        .find(|stage| {
            stage.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && stage.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.profile_overrepresented_sequences")
        })
        .expect("profile overrepresented stage");
    assert_eq!(
        profile_overrepresented.get("blocked_tool_count").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(
        profile_overrepresented
            .get("shared_metric_field_count")
            .and_then(serde_json::Value::as_u64),
        Some(3)
    );

    let bam_damage = stages
        .iter()
        .find(|stage| {
            stage.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.damage")
        })
        .expect("bam damage stage");
    assert_eq!(bam_damage.get("tool_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(
        bam_damage.get("comparison_contract_status").and_then(serde_json::Value::as_str),
        Some("declared")
    );
    assert_eq!(
        bam_damage.get("shared_metric_field_count").and_then(serde_json::Value::as_u64),
        Some(5)
    );
}
