#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
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

    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn bench_local_vcf_ibd_smoke_reports_pair_rows_and_localized_probe() {
    let payload = run_cli_json(&["bench", "local", "run-vcf-ibd-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_ibd_smoke.v1")
    );
    assert_eq!(
        payload.get("command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-ibd-smoke --tool-id germline")
    );
    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.ibd"));
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("germline"));
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(
        payload.get("input_fixture_id").and_then(serde_json::Value::as_str),
        Some("vcf_mini_multisample_cohort")
    );
    assert_eq!(
        payload.get("output_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.ibd/germline")
    );
    assert_eq!(
        payload.get("ibd_tsv_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.ibd/germline/ibd.tsv")
    );
    assert_eq!(
        payload.get("ibd_json_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.ibd/germline/ibd.json")
    );
    assert_eq!(payload.get("status").and_then(serde_json::Value::as_str), Some("complete"));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    assert!(!rows.is_empty(), "expected at least one IBD row");
    assert!(rows.iter().all(|row| {
        row.get("sample_a").and_then(serde_json::Value::as_str).is_some()
            && row.get("sample_b").and_then(serde_json::Value::as_str).is_some()
            && row.get("segment_count").and_then(serde_json::Value::as_u64).unwrap_or(0) > 0
            && row.get("total_length").and_then(serde_json::Value::as_f64).unwrap_or(0.0) > 0.0
            && row.get("overlap_marker_count").and_then(serde_json::Value::as_u64).unwrap_or(0) > 0
            && row.get("status").and_then(serde_json::Value::as_str) == Some("complete")
    }));

    let probe = payload.get("insufficient_overlap_probe").expect("insufficient overlap probe");
    assert_eq!(
        probe.get("ibd_status").and_then(serde_json::Value::as_str),
        Some("insufficient_marker_overlap")
    );
    assert_eq!(
        probe.get("insufficient_data_reason").and_then(serde_json::Value::as_str),
        Some("no_pairs_met_min_marker_or_length_threshold")
    );
    assert_eq!(probe.get("filtered_segment_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        probe.get("unrelated_stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.roh")
    );
    assert_eq!(
        probe.get("unrelated_stage_status").and_then(serde_json::Value::as_str),
        Some("complete")
    );
    assert!(
        probe.get("unrelated_stage_segment_count").and_then(serde_json::Value::as_u64).unwrap_or(0)
            > 0
    );

    let repo_root = support::repo_root().expect("repo root");
    let persisted_path = repo_root.join("runs/bench/local-smoke/vcf.ibd/germline/ibd.json");
    let persisted_raw = std::fs::read_to_string(&persisted_path).expect("read persisted report");
    let persisted: serde_json::Value = serde_json::from_str(&persisted_raw).expect("parse report");
    assert_eq!(
        persisted.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_ibd_smoke.v1")
    );
}
