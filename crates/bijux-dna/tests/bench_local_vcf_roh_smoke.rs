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
fn bench_local_vcf_roh_smoke_reports_normalized_segments_and_summary() {
    let payload = run_cli_json(&["bench", "local", "run-vcf-roh-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_roh_smoke.v1")
    );
    assert_eq!(
        payload.get("command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-roh-smoke --tool-id plink2")
    );
    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.roh"));
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("plink2"));
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
        Some("target/local-smoke/vcf.roh/plink2")
    );
    assert_eq!(
        payload.get("roh_tsv_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.roh/plink2/roh.tsv")
    );
    assert_eq!(
        payload.get("roh_json_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.roh/plink2/roh.json")
    );
    assert_eq!(
        payload.get("source_roh_segments_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.roh/plink2/source_roh_segments.tsv")
    );
    assert_eq!(
        payload.get("source_roh_per_sample_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.roh/plink2/source_roh_per_sample.tsv")
    );
    assert_eq!(
        payload.get("source_roh_report_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.roh/plink2/source_roh.json")
    );
    assert_eq!(
        payload.get("stage_result_manifest_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.roh/plink2/stage-result.json")
    );
    assert_eq!(payload.get("status").and_then(serde_json::Value::as_str), Some("complete"));

    let segments = payload.get("segments").and_then(serde_json::Value::as_array).expect("segments");
    assert!(!segments.is_empty(), "expected at least one ROH segment");
    assert!(segments.iter().all(|row| {
        row.get("sample_id").and_then(serde_json::Value::as_str).is_some()
            && row.get("contig").and_then(serde_json::Value::as_str).is_some()
            && row.get("start").and_then(serde_json::Value::as_u64).is_some()
            && row.get("end").and_then(serde_json::Value::as_u64).is_some()
            && row.get("length").and_then(serde_json::Value::as_u64).is_some()
            && row.get("variant_count").and_then(serde_json::Value::as_u64).is_some()
    }));

    let per_sample_summary = payload
        .get("per_sample_summary")
        .and_then(serde_json::Value::as_array)
        .expect("per sample summary");
    assert_eq!(per_sample_summary.len(), 4);
    assert_eq!(
        per_sample_summary
            .iter()
            .filter_map(|row| row.get("sample_id").and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>(),
        vec!["sample_a", "sample_b", "sample_c", "sample_d"]
    );
    assert_eq!(
        per_sample_summary
            .iter()
            .filter_map(|row| row.get("segment_count").and_then(serde_json::Value::as_u64))
            .sum::<u64>(),
        payload.get("segment_count").and_then(serde_json::Value::as_u64).expect("segment count")
    );
    assert_eq!(
        per_sample_summary
            .iter()
            .filter_map(|row| row.get("total_length").and_then(serde_json::Value::as_u64))
            .sum::<u64>(),
        payload.get("total_length").and_then(serde_json::Value::as_u64).expect("total length")
    );

    let repo_root = support::repo_root().expect("repo root");
    let persisted_path = repo_root.join("target/local-smoke/vcf.roh/plink2/roh.json");
    let persisted_raw = std::fs::read_to_string(&persisted_path).expect("read persisted report");
    let persisted: serde_json::Value = serde_json::from_str(&persisted_raw).expect("parse report");
    assert_eq!(
        persisted.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_roh_smoke.v1")
    );
    assert_eq!(
        persisted
            .get("per_sample_summary")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.len()),
        Some(4)
    );
}
