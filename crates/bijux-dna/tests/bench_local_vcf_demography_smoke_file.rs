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
fn bench_local_vcf_demography_smoke_writes_governed_files() {
    let output = run_cli(&["bench", "local", "run-vcf-demography-smoke"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "runs/bench/local-smoke/vcf.demography/ibdne/demography.json"
    );

    let repo_root = support::repo_root().expect("repo root");
    let report_path = repo_root.join("runs/bench/local-smoke/vcf.demography/ibdne/demography.json");
    let source_upstream_report_path =
        repo_root.join("runs/bench/local-smoke/vcf.demography/ibdne/source_ibd_smoke.json");
    let source_upstream_filtered_segments_path = repo_root
        .join("runs/bench/local-smoke/vcf.demography/ibdne/source_ibd_filtered_segments.tsv");
    let source_ne_trajectory_path =
        repo_root.join("runs/bench/local-smoke/vcf.demography/ibdne/source_ne_trajectory.tsv");
    let source_demography_contract_path = repo_root
        .join("runs/bench/local-smoke/vcf.demography/ibdne/source_demography_contract.json");
    let source_demography_metrics_path = repo_root
        .join("runs/bench/local-smoke/vcf.demography/ibdne/source_demography_metrics.json");
    let source_logs_path =
        repo_root.join("runs/bench/local-smoke/vcf.demography/ibdne/source_logs.txt");
    let probe_input_ibd_path = repo_root.join(
        "runs/bench/local-smoke/vcf.demography/ibdne/artifacts/probe/probe_input_ibd_segments.tsv",
    );
    let probe_ne_trajectory_path = repo_root.join(
        "runs/bench/local-smoke/vcf.demography/ibdne/artifacts/probe/probe_source_ne_trajectory.tsv",
    );
    let probe_demography_contract_path = repo_root.join(
        "runs/bench/local-smoke/vcf.demography/ibdne/artifacts/probe/probe_source_demography_contract.json",
    );
    let probe_demography_metrics_path = repo_root.join(
        "runs/bench/local-smoke/vcf.demography/ibdne/artifacts/probe/probe_source_demography_metrics.json",
    );
    let probe_logs_path = repo_root
        .join("runs/bench/local-smoke/vcf.demography/ibdne/artifacts/probe/probe_source_logs.txt");
    let stage_result_path =
        repo_root.join("runs/bench/local-smoke/vcf.demography/ibdne/stage-result.json");

    for path in [
        &report_path,
        &source_upstream_report_path,
        &source_upstream_filtered_segments_path,
        &source_ne_trajectory_path,
        &source_demography_contract_path,
        &source_demography_metrics_path,
        &source_logs_path,
        &probe_input_ibd_path,
        &probe_ne_trajectory_path,
        &probe_demography_contract_path,
        &probe_demography_metrics_path,
        &probe_logs_path,
        &stage_result_path,
    ] {
        assert!(path.is_file(), "expected file at {}", path.display());
    }

    let report_raw = std::fs::read_to_string(&report_path).expect("read report");
    let report: serde_json::Value = serde_json::from_str(&report_raw).expect("parse report");
    assert_eq!(report.get("status").and_then(serde_json::Value::as_str), Some("complete"));
    assert!(report
        .get("ne_estimates")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|rows| !rows.is_empty()));

    let trajectory_tsv = std::fs::read_to_string(&source_ne_trajectory_path).expect("read tsv");
    assert_eq!(trajectory_tsv.lines().next(), Some("generation\tne\tci_low\tci_high"));

    let probe_contract_raw =
        std::fs::read_to_string(&probe_demography_contract_path).expect("read probe contract");
    let probe_contract: serde_json::Value =
        serde_json::from_str(&probe_contract_raw).expect("parse probe contract");
    assert_eq!(
        probe_contract.get("status").and_then(serde_json::Value::as_str),
        Some("insufficient_data")
    );
    assert_eq!(
        probe_contract.get("insufficient_data_reason").and_then(serde_json::Value::as_str),
        Some("not_enough_ibd_segments")
    );

    let manifest_raw = std::fs::read_to_string(&stage_result_path).expect("read stage result");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw).expect("parse manifest");
    assert_eq!(
        manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result.v2")
    );
    assert_eq!(
        manifest.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.demography")
    );
    let outputs = manifest.get("outputs").and_then(serde_json::Value::as_array).expect("outputs");
    assert_eq!(outputs.len(), 12);
}
