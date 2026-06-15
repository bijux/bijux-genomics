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
fn bench_local_vcf_filter_smoke_writes_governed_files() {
    let output = run_cli(&["bench", "local", "run-vcf-filter-smoke"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "runs/bench/local-smoke/vcf.filter/bcftools/filtered.vcf.gz"
    );

    let repo_root = support::repo_root().expect("repo root");
    let output_vcf = repo_root.join("runs/bench/local-smoke/vcf.filter/bcftools/filtered.vcf.gz");
    let output_tbi =
        repo_root.join("runs/bench/local-smoke/vcf.filter/bcftools/filtered.vcf.gz.tbi");
    let metrics_path = repo_root.join("runs/bench/local-smoke/vcf.filter/bcftools/metrics.json");
    let filter_breakdown_path =
        repo_root.join("runs/bench/local-smoke/vcf.filter/bcftools/filter_breakdown.json");
    let filter_breakdown_tsv_path =
        repo_root.join("runs/bench/local-smoke/vcf.filter/bcftools/filter_breakdown.tsv");
    let filter_explain_path =
        repo_root.join("runs/bench/local-smoke/vcf.filter/bcftools/filter_explain.json");
    let manifest_path =
        repo_root.join("runs/bench/local-smoke/vcf.filter/bcftools/stage-result.json");
    let input_vcf = repo_root
        .join("runs/bench/local-smoke/vcf.filter/bcftools/artifacts/input/filter_input.vcf");

    assert!(output_vcf.is_file(), "expected output VCF at {}", output_vcf.display());
    assert!(output_tbi.is_file(), "expected output index at {}", output_tbi.display());
    assert!(metrics_path.is_file(), "expected metrics at {}", metrics_path.display());
    assert!(
        filter_breakdown_path.is_file(),
        "expected breakdown json at {}",
        filter_breakdown_path.display()
    );
    assert!(
        filter_breakdown_tsv_path.is_file(),
        "expected breakdown tsv at {}",
        filter_breakdown_tsv_path.display()
    );
    assert!(
        filter_explain_path.is_file(),
        "expected explain json at {}",
        filter_explain_path.display()
    );
    assert!(manifest_path.is_file(), "expected stage result at {}", manifest_path.display());
    assert!(input_vcf.is_file(), "expected synthetic input at {}", input_vcf.display());

    let breakdown_raw = std::fs::read_to_string(&filter_breakdown_path).expect("read breakdown");
    let breakdown: serde_json::Value =
        serde_json::from_str(&breakdown_raw).expect("parse breakdown");
    assert_eq!(breakdown.pointer("/counts/PASS").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(breakdown.pointer("/counts/LOWQUAL").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(breakdown.pointer("/counts/LOW_DP").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(breakdown.pointer("/counts/LOW_MQ").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        breakdown.pointer("/counts/HIGH_MISSING").and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let explain_raw = std::fs::read_to_string(&filter_explain_path).expect("read explain");
    let explain: serde_json::Value = serde_json::from_str(&explain_raw).expect("parse explain");
    assert_eq!(
        explain.pointer("/filter_scope/output_subset").and_then(serde_json::Value::as_str),
        Some("retain_tagged_records")
    );
    assert_eq!(
        explain.pointer("/thresholds/min_qual").and_then(serde_json::Value::as_f64),
        Some(30.0)
    );

    let manifest_raw = std::fs::read_to_string(&manifest_path).expect("read manifest");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw).expect("parse manifest");
    assert_eq!(
        manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result.v2")
    );
    assert_eq!(manifest.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.filter"));
    assert_eq!(
        manifest.get("tool").and_then(|value| value.get("id")).and_then(serde_json::Value::as_str),
        Some("bcftools")
    );
    assert_eq!(
        manifest
            .get("command")
            .and_then(|value| value.get("rendered"))
            .and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-filter-smoke --tool-id bcftools")
    );

    let outputs =
        manifest.get("outputs").and_then(serde_json::Value::as_array).expect("outputs array");
    assert_eq!(outputs.len(), 6);
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("filtered_vcf")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("runs/bench/local-smoke/vcf.filter/bcftools/filtered.vcf.gz")
    }));
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("filter_explain_json")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("runs/bench/local-smoke/vcf.filter/bcftools/filter_explain.json")
    }));
}
