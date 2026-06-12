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

#[test]
fn bench_readiness_stage_tool_benchmark_ready_writes_json_output() {
    let output = run_cli(&["bench", "readiness", "render-stage-tool-benchmark-ready"]);
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
        "benchmarks/readiness/FASTQ_BAM_STAGE_TOOL_BENCHMARK_READY.json"
    );

    let repo_root = support::repo_root().expect("repo root");
    let json_payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read stage-tool benchmark-ready json");
    let json_value: serde_json::Value =
        serde_json::from_str(&json_payload).expect("parse stage-tool benchmark-ready json");

    assert_eq!(
        json_value.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.fastq_bam_stage_tool_benchmark_ready.v1")
    );
    assert_eq!(json_value.get("passes_gate").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        json_value.get("benchmark_ready_pair_count").and_then(serde_json::Value::as_u64),
        Some(118)
    );
    assert_eq!(json_value.get("excluded_pair_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(json_value.get("failing_pair_count").and_then(serde_json::Value::as_u64), Some(0));

    let excluded_pairs = json_value
        .get("excluded_pairs")
        .and_then(serde_json::Value::as_array)
        .expect("excluded pairs");
    assert!(excluded_pairs.iter().any(|row| {
        row.get("row_id").and_then(serde_json::Value::as_str)
            == Some("fastq:fastq.report_qc:multiqc")
            && row.get("excluded_from_generated_jobs").and_then(serde_json::Value::as_bool)
                == Some(true)
            && row.get("excluded_from_expected_results").and_then(serde_json::Value::as_bool)
                == Some(true)
    }));

    let surfaces = json_value
        .get("surface_summaries")
        .and_then(serde_json::Value::as_array)
        .expect("surface summaries");
    assert!(surfaces.iter().any(|surface| {
        surface.get("surface_id").and_then(serde_json::Value::as_str) == Some("expected_results")
            && surface.get("covered_count").and_then(serde_json::Value::as_u64) == Some(118)
            && surface.get("excluded_count").and_then(serde_json::Value::as_u64) == Some(5)
    }));
}
