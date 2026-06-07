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
fn bench_local_vcf_imputation_metrics_smoke_writes_governed_files() {
    let output = run_cli(&["bench", "local", "run-vcf-imputation-metrics-smoke"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "runs/bench/local-smoke/vcf.imputation_metrics/beagle/imputation_metrics.json"
    );

    let repo_root = support::repo_root().expect("repo root");
    let report_path = repo_root
        .join("runs/bench/local-smoke/vcf.imputation_metrics/beagle/imputation_metrics.json");
    let source_imputation_qc_path = repo_root
        .join("runs/bench/local-smoke/vcf.imputation_metrics/beagle/source_imputation_qc.json");
    let source_impute_smoke_metrics_path = repo_root.join(
        "runs/bench/local-smoke/vcf.imputation_metrics/beagle/source_impute_smoke_metrics.json",
    );
    let source_imputation_manifest_path = repo_root.join(
        "runs/bench/local-smoke/vcf.imputation_metrics/beagle/source_imputation_manifest.json",
    );
    let stage_result_path =
        repo_root.join("runs/bench/local-smoke/vcf.imputation_metrics/beagle/stage-result.json");

    assert!(report_path.is_file(), "expected report at {}", report_path.display());
    assert!(
        source_imputation_qc_path.is_file(),
        "expected source qc at {}",
        source_imputation_qc_path.display()
    );
    assert!(
        source_impute_smoke_metrics_path.is_file(),
        "expected source smoke metrics at {}",
        source_impute_smoke_metrics_path.display()
    );
    assert!(
        source_imputation_manifest_path.is_file(),
        "expected source manifest at {}",
        source_imputation_manifest_path.display()
    );
    assert!(
        stage_result_path.is_file(),
        "expected stage result at {}",
        stage_result_path.display()
    );

    let report_raw = std::fs::read_to_string(&report_path).expect("read report");
    let report: serde_json::Value = serde_json::from_str(&report_raw).expect("parse report");
    assert_eq!(report.get("status").and_then(serde_json::Value::as_str), Some("complete"));
    assert_eq!(report.get("concordance").and_then(serde_json::Value::as_f64), Some(1.0));
    assert_eq!(report.get("r2_available").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(report.get("low_confidence_sites").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(report.get("masked_truth_sites").and_then(serde_json::Value::as_u64), Some(1));

    let source_qc_raw =
        std::fs::read_to_string(&source_imputation_qc_path).expect("read source qc");
    let source_qc: serde_json::Value =
        serde_json::from_str(&source_qc_raw).expect("parse source qc");
    assert_eq!(source_qc.get("backend").and_then(serde_json::Value::as_str), Some("beagle"));
    assert_eq!(source_qc.get("variant_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        source_qc
            .pointer("/concordance/masked_truth_site_count")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(source_qc.get("low_confidence_count").and_then(serde_json::Value::as_u64), Some(1));

    let source_metrics_raw =
        std::fs::read_to_string(&source_impute_smoke_metrics_path).expect("read source metrics");
    let source_metrics: serde_json::Value =
        serde_json::from_str(&source_metrics_raw).expect("parse source metrics");
    assert_eq!(source_metrics.get("variant_count").and_then(serde_json::Value::as_u64), Some(2));

    let manifest_raw = std::fs::read_to_string(&stage_result_path).expect("read stage result");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw).expect("parse manifest");
    assert_eq!(
        manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result.v2")
    );
    assert_eq!(
        manifest.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.imputation_metrics")
    );
    assert_eq!(
        manifest.get("tool").and_then(|value| value.get("id")).and_then(serde_json::Value::as_str),
        Some("beagle")
    );
    assert_eq!(
        manifest
            .get("command")
            .and_then(|value| value.get("rendered"))
            .and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-imputation-metrics-smoke --tool-id beagle")
    );

    let outputs =
        manifest.get("outputs").and_then(serde_json::Value::as_array).expect("outputs array");
    assert_eq!(outputs.len(), 4);
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str)
            == Some("imputation_metrics_json")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some(
                    "runs/bench/local-smoke/vcf.imputation_metrics/beagle/imputation_metrics.json",
                )
    }));
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str)
            == Some("source_imputation_qc_json")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some(
                    "runs/bench/local-smoke/vcf.imputation_metrics/beagle/source_imputation_qc.json",
                )
    }));
}
