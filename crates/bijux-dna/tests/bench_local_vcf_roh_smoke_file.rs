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
fn bench_local_vcf_roh_smoke_writes_governed_files() {
    let output = run_cli(&["bench", "local", "run-vcf-roh-smoke"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "runs/bench/local-smoke/vcf.roh/plink2/roh.json"
    );

    let repo_root = support::repo_root().expect("repo root");
    let report_path = repo_root.join("runs/bench/local-smoke/vcf.roh/plink2/roh.json");
    let roh_tsv_path = repo_root.join("runs/bench/local-smoke/vcf.roh/plink2/roh.tsv");
    let source_segments_path =
        repo_root.join("runs/bench/local-smoke/vcf.roh/plink2/source_roh_segments.tsv");
    let source_per_sample_path =
        repo_root.join("runs/bench/local-smoke/vcf.roh/plink2/source_roh_per_sample.tsv");
    let source_report_path =
        repo_root.join("runs/bench/local-smoke/vcf.roh/plink2/source_roh.json");
    let source_metrics_path =
        repo_root.join("runs/bench/local-smoke/vcf.roh/plink2/source_metrics.json");
    let source_summary_path =
        repo_root.join("runs/bench/local-smoke/vcf.roh/plink2/source_roh_summary.json");
    let source_roh_metrics_path =
        repo_root.join("runs/bench/local-smoke/vcf.roh/plink2/source_roh_metrics.json");
    let source_logs_path = repo_root.join("runs/bench/local-smoke/vcf.roh/plink2/source_logs.txt");
    let stage_result_path =
        repo_root.join("runs/bench/local-smoke/vcf.roh/plink2/stage-result.json");

    for path in [
        &report_path,
        &roh_tsv_path,
        &source_segments_path,
        &source_per_sample_path,
        &source_report_path,
        &source_metrics_path,
        &source_summary_path,
        &source_roh_metrics_path,
        &source_logs_path,
        &stage_result_path,
    ] {
        assert!(path.is_file(), "expected file at {}", path.display());
    }

    let report_raw = std::fs::read_to_string(&report_path).expect("read report");
    let report: serde_json::Value = serde_json::from_str(&report_raw).expect("parse report");
    assert_eq!(report.get("status").and_then(serde_json::Value::as_str), Some("complete"));
    assert_eq!(report.get("sample_count").and_then(serde_json::Value::as_u64), Some(4));
    assert!(
        report.get("segment_count").and_then(serde_json::Value::as_u64).unwrap_or(0) > 0,
        "expected at least one normalized ROH segment"
    );

    let roh_tsv = std::fs::read_to_string(&roh_tsv_path).expect("read roh tsv");
    assert_eq!(
        roh_tsv.lines().next(),
        Some("sample_id\tcontig\tstart\tend\tlength\tvariant_count")
    );

    let source_segments =
        std::fs::read_to_string(&source_segments_path).expect("read source segments");
    assert_eq!(
        source_segments.lines().next(),
        Some("sample\tcontig\tstart\tend\tlength_bp\tn_sites")
    );

    let source_per_sample =
        std::fs::read_to_string(&source_per_sample_path).expect("read source per sample");
    assert_eq!(
        source_per_sample.lines().next(),
        Some("sample\tsegment_count\ttotal_length_bp\tmean_length_bp")
    );

    let source_report_raw =
        std::fs::read_to_string(&source_report_path).expect("read source report");
    let source_report: serde_json::Value =
        serde_json::from_str(&source_report_raw).expect("parse source report");
    assert_eq!(
        source_report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.vcf.roh.summary.v2")
    );
    assert!(matches!(
        source_report.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("real_tool" | "fallback_proxy")
    ));

    let manifest_raw = std::fs::read_to_string(&stage_result_path).expect("read stage result");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw).expect("parse manifest");
    assert_eq!(
        manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result.v2")
    );
    assert_eq!(manifest.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.roh"));
    assert_eq!(
        manifest.get("tool").and_then(|value| value.get("id")).and_then(serde_json::Value::as_str),
        Some("plink2")
    );
    let outputs = manifest.get("outputs").and_then(serde_json::Value::as_array).expect("outputs");
    assert_eq!(outputs.len(), 9);
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("roh_json")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("runs/bench/local-smoke/vcf.roh/plink2/roh.json")
    }));
}
