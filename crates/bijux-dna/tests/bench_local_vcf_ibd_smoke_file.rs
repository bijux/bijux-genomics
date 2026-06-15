#![allow(clippy::expect_used, clippy::too_many_lines)]

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
fn bench_local_vcf_ibd_smoke_writes_governed_files() {
    let output = run_cli(&["bench", "local", "run-vcf-ibd-smoke"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "runs/bench/local-smoke/vcf.ibd/germline/ibd.json"
    );

    let repo_root = support::repo_root().expect("repo root");
    let report_path = repo_root.join("runs/bench/local-smoke/vcf.ibd/germline/ibd.json");
    let ibd_tsv_path = repo_root.join("runs/bench/local-smoke/vcf.ibd/germline/ibd.tsv");
    let source_input_path =
        repo_root.join("runs/bench/local-smoke/vcf.ibd/germline/source_ibd_input.tsv");
    let source_segments_path =
        repo_root.join("runs/bench/local-smoke/vcf.ibd/germline/source_ibd_segments.tsv");
    let source_merged_segments_path =
        repo_root.join("runs/bench/local-smoke/vcf.ibd/germline/source_ibd_merged_segments.tsv");
    let source_filtered_segments_path =
        repo_root.join("runs/bench/local-smoke/vcf.ibd/germline/source_ibd_filtered_segments.tsv");
    let source_summary_path =
        repo_root.join("runs/bench/local-smoke/vcf.ibd/germline/source_ibd_summary.json");
    let source_metrics_path =
        repo_root.join("runs/bench/local-smoke/vcf.ibd/germline/source_ibd_metrics.json");
    let source_logs_path =
        repo_root.join("runs/bench/local-smoke/vcf.ibd/germline/source_logs.txt");
    let probe_summary_path = repo_root.join(
        "runs/bench/local-smoke/vcf.ibd/germline/artifacts/probe/probe_source_ibd_summary.json",
    );
    let probe_filtered_path = repo_root
        .join("runs/bench/local-smoke/vcf.ibd/germline/artifacts/probe/probe_source_ibd_filtered_segments.tsv");
    let probe_roh_summary_path = repo_root.join(
        "runs/bench/local-smoke/vcf.ibd/germline/artifacts/probe/probe_source_roh_summary.json",
    );
    let probe_roh_segments_path = repo_root.join(
        "runs/bench/local-smoke/vcf.ibd/germline/artifacts/probe/probe_source_roh_segments.tsv",
    );
    let stage_result_path =
        repo_root.join("runs/bench/local-smoke/vcf.ibd/germline/stage-result.json");

    for path in [
        &report_path,
        &ibd_tsv_path,
        &source_input_path,
        &source_segments_path,
        &source_merged_segments_path,
        &source_filtered_segments_path,
        &source_summary_path,
        &source_metrics_path,
        &source_logs_path,
        &probe_summary_path,
        &probe_filtered_path,
        &probe_roh_summary_path,
        &probe_roh_segments_path,
        &stage_result_path,
    ] {
        assert!(path.is_file(), "expected file at {}", path.display());
    }

    let report_raw = std::fs::read_to_string(&report_path).expect("read report");
    let report: serde_json::Value = serde_json::from_str(&report_raw).expect("parse report");
    assert_eq!(report.get("status").and_then(serde_json::Value::as_str), Some("complete"));
    assert!(report.get("pair_count").and_then(serde_json::Value::as_u64).unwrap_or(0) > 0);

    let ibd_tsv = std::fs::read_to_string(&ibd_tsv_path).expect("read ibd tsv");
    assert_eq!(
        ibd_tsv.lines().next(),
        Some("sample_a\tsample_b\tsegment_count\ttotal_length\toverlap_marker_count\tstatus")
    );

    let probe_summary_raw =
        std::fs::read_to_string(&probe_summary_path).expect("read probe summary");
    let probe_summary: serde_json::Value =
        serde_json::from_str(&probe_summary_raw).expect("parse probe summary");
    assert_eq!(
        probe_summary.get("status").and_then(serde_json::Value::as_str),
        Some("insufficient_marker_overlap")
    );

    let manifest_raw = std::fs::read_to_string(&stage_result_path).expect("read stage result");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw).expect("parse manifest");
    assert_eq!(
        manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result.v2")
    );
    assert_eq!(manifest.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.ibd"));
    let outputs = manifest.get("outputs").and_then(serde_json::Value::as_array).expect("outputs");
    assert_eq!(outputs.len(), 13);
}
