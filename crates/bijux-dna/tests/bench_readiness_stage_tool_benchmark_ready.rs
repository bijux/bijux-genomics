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
fn bench_readiness_stage_tool_benchmark_ready_tracks_ready_slice_and_excluded_pairs() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-stage-tool-benchmark-ready", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.fastq_bam_stage_tool_benchmark_ready.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/FASTQ_BAM_STAGE_TOOL_BENCHMARK_READY.json")
    );
    assert_eq!(payload.get("passes_gate").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("expected_pair_count").and_then(serde_json::Value::as_u64), Some(123));
    assert_eq!(
        payload.get("benchmark_ready_pair_count").and_then(serde_json::Value::as_u64),
        Some(118)
    );
    assert_eq!(payload.get("excluded_pair_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("failing_pair_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("generated_job_pair_count").and_then(serde_json::Value::as_u64),
        Some(118)
    );
    assert_eq!(
        payload.get("expected_result_pair_count").and_then(serde_json::Value::as_u64),
        Some(118)
    );
    assert_eq!(
        payload.get("benchmark_ready_stage_count").and_then(serde_json::Value::as_u64),
        Some(50)
    );
    assert_eq!(
        payload.get("excluded_registry_gap_count").and_then(serde_json::Value::as_u64),
        Some(4)
    );

    let surfaces = payload
        .get("surface_summaries")
        .and_then(serde_json::Value::as_array)
        .expect("surface summaries");
    assert_eq!(surfaces.len(), 8);
    assert!(surfaces.iter().any(|surface| {
        surface.get("surface_id").and_then(serde_json::Value::as_str) == Some("tool_registry")
            && surface.get("surface_status").and_then(serde_json::Value::as_str)
                == Some("ready_slice_complete")
            && surface.get("excluded_count").and_then(serde_json::Value::as_u64) == Some(4)
            && surface.get("failing_count").and_then(serde_json::Value::as_u64) == Some(0)
    }));
    assert!(surfaces.iter().any(|surface| {
        surface.get("surface_id").and_then(serde_json::Value::as_str) == Some("command_adapters")
            && surface.get("covered_count").and_then(serde_json::Value::as_u64) == Some(118)
            && surface.get("excluded_count").and_then(serde_json::Value::as_u64) == Some(5)
    }));

    let failing_pairs =
        payload.get("failing_pairs").and_then(serde_json::Value::as_array).expect("failing pairs");
    assert!(
        failing_pairs.is_empty(),
        "ready slice must pass without failing benchmark-ready pairs"
    );

    let excluded_pairs = payload
        .get("excluded_pairs")
        .and_then(serde_json::Value::as_array)
        .expect("excluded pairs");
    assert_eq!(excluded_pairs.len(), 5);
    assert!(excluded_pairs.iter().any(|row| {
        row.get("row_id").and_then(serde_json::Value::as_str)
            == Some("fastq:fastq.trim_reads:seqpurge")
            && row.get("registry_status").and_then(serde_json::Value::as_str)
                == Some("tool_missing")
            && row.get("excluded_from_generated_jobs").and_then(serde_json::Value::as_bool)
                == Some(true)
            && row.get("excluded_from_expected_results").and_then(serde_json::Value::as_bool)
                == Some(true)
    }));
    assert!(
        excluded_pairs.iter().all(|row| {
            row.get("row_id")
                .and_then(serde_json::Value::as_str)
                .is_none_or(|row_id| !row_id.starts_with("fastq:fastq.index_reference:"))
        }),
        "asset-backed index-reference pairs must stay out of the excluded slice once corpus assignments are governed"
    );
}
