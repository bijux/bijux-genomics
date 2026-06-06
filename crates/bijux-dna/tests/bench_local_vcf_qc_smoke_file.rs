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
fn bench_local_vcf_qc_smoke_writes_governed_files() {
    let output = run_cli(&["bench", "local", "run-vcf-qc-smoke"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "target/local-smoke/vcf.qc/plink2/qc.json"
    );

    let repo_root = support::repo_root().expect("repo root");
    let qc_json_path = repo_root.join("target/local-smoke/vcf.qc/plink2/qc.json");
    let qc_summary_path = repo_root.join("target/local-smoke/vcf.qc/plink2/qc_summary.json");
    let qc_tables_path = repo_root.join("target/local-smoke/vcf.qc/plink2/qc_tables.tsv");
    let imputation_qc_path = repo_root.join("target/local-smoke/vcf.qc/plink2/imputation_qc.tsv");
    let warnings_path = repo_root.join("target/local-smoke/vcf.qc/plink2/warnings.json");
    let qc_histograms_path = repo_root.join("target/local-smoke/vcf.qc/plink2/qc_histograms.json");
    let metrics_path = repo_root.join("target/local-smoke/vcf.qc/plink2/metrics.json");
    let manifest_path = repo_root.join("target/local-smoke/vcf.qc/plink2/stage-result.json");
    let input_vcf_path =
        repo_root.join("target/local-smoke/vcf.qc/plink2/artifacts/input/qc_input.vcf");

    assert!(qc_json_path.is_file(), "expected QC report at {}", qc_json_path.display());
    assert!(qc_summary_path.is_file(), "expected QC summary at {}", qc_summary_path.display());
    assert!(qc_tables_path.is_file(), "expected QC table at {}", qc_tables_path.display());
    assert!(
        imputation_qc_path.is_file(),
        "expected imputation QC table at {}",
        imputation_qc_path.display()
    );
    assert!(warnings_path.is_file(), "expected warnings at {}", warnings_path.display());
    assert!(
        qc_histograms_path.is_file(),
        "expected histograms at {}",
        qc_histograms_path.display()
    );
    assert!(metrics_path.is_file(), "expected metrics at {}", metrics_path.display());
    assert!(manifest_path.is_file(), "expected manifest at {}", manifest_path.display());
    assert!(input_vcf_path.is_file(), "expected input VCF at {}", input_vcf_path.display());

    let summary_raw = std::fs::read_to_string(&qc_summary_path).expect("read QC summary");
    let summary: serde_json::Value = serde_json::from_str(&summary_raw).expect("parse QC summary");
    assert_eq!(
        summary.get("sample_missingness_exclusion_threshold").and_then(serde_json::Value::as_f64),
        Some(0.5)
    );
    let excluded_samples = summary
        .get("excluded_samples")
        .and_then(serde_json::Value::as_array)
        .expect("excluded sample rows");
    assert_eq!(excluded_samples.len(), 1);
    assert_eq!(
        excluded_samples[0].get("sample_id").and_then(serde_json::Value::as_str),
        Some("qc_sparse")
    );
    let excluded_variants = summary
        .get("excluded_variants")
        .and_then(serde_json::Value::as_array)
        .expect("excluded variant rows");
    assert_eq!(excluded_variants.len(), 1);
    assert_eq!(
        excluded_variants[0].get("variant_id").and_then(serde_json::Value::as_str),
        Some("chr1:30:G:A")
    );

    let metrics_raw = std::fs::read_to_string(&metrics_path).expect("read metrics");
    let metrics: serde_json::Value = serde_json::from_str(&metrics_raw).expect("parse metrics");
    assert_eq!(
        metrics.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_qc_smoke.metrics.v1")
    );
    assert_eq!(
        metrics.get("sample_missingness_exclusion_threshold").and_then(serde_json::Value::as_f64),
        Some(0.5)
    );

    let manifest_raw = std::fs::read_to_string(&manifest_path).expect("read manifest");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw).expect("parse manifest");
    assert_eq!(
        manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result.v2")
    );
    assert_eq!(manifest.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.qc"));
    assert_eq!(
        manifest.get("tool").and_then(|value| value.get("id")).and_then(serde_json::Value::as_str),
        Some("plink2")
    );
    assert_eq!(
        manifest
            .get("command")
            .and_then(|value| value.get("rendered"))
            .and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-qc-smoke --tool-id plink2")
    );

    let outputs =
        manifest.get("outputs").and_then(serde_json::Value::as_array).expect("outputs array");
    assert_eq!(outputs.len(), 7);
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("qc_report_json")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("target/local-smoke/vcf.qc/plink2/qc.json")
    }));
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("qc_summary_json")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("target/local-smoke/vcf.qc/plink2/qc_summary.json")
    }));
}
