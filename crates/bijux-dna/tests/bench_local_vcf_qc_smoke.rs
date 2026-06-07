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
fn bench_local_vcf_qc_smoke_reports_real_governed_outputs() {
    let payload = run_cli_json(&["bench", "local", "run-vcf-qc-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_qc_smoke.v1")
    );
    assert_eq!(
        payload.get("command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-qc-smoke --tool-id plink2")
    );
    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.qc"));
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("plink2"));
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(
        payload.get("input_fixture_id").and_then(serde_json::Value::as_str),
        Some("qc_cohort_missingness")
    );
    assert_eq!(payload.get("sample_name").and_then(serde_json::Value::as_str), Some("qc_cohort"));
    assert_eq!(
        payload.get("input_vcf_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.qc/plink2/artifacts/input/qc_input.vcf")
    );
    assert_eq!(
        payload.get("output_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.qc/plink2")
    );
    assert_eq!(
        payload.get("qc_json_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.qc/plink2/qc.json")
    );
    assert_eq!(
        payload.get("qc_summary_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.qc/plink2/qc_summary.json")
    );
    assert_eq!(
        payload.get("metrics_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.qc/plink2/metrics.json")
    );
    assert_eq!(
        payload.get("stage_result_manifest_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.qc/plink2/stage-result.json")
    );
    assert_eq!(payload.get("exit_code").and_then(serde_json::Value::as_i64), Some(0));
    assert_eq!(
        payload.get("sample_missingness_exclusion_threshold").and_then(serde_json::Value::as_f64),
        Some(0.5)
    );
    assert_eq!(
        payload.get("variant_missingness_exclusion_threshold").and_then(serde_json::Value::as_f64),
        Some(0.5)
    );

    let excluded_samples = payload
        .get("excluded_samples")
        .and_then(serde_json::Value::as_array)
        .expect("excluded_samples array");
    assert_eq!(excluded_samples.len(), 1);
    assert_eq!(
        excluded_samples[0].get("sample_id").and_then(serde_json::Value::as_str),
        Some("qc_sparse")
    );
    assert_eq!(
        excluded_samples[0].get("missingness").and_then(serde_json::Value::as_f64),
        Some(0.75)
    );

    let excluded_variants = payload
        .get("excluded_variants")
        .and_then(serde_json::Value::as_array)
        .expect("excluded_variants array");
    assert_eq!(excluded_variants.len(), 1);
    assert_eq!(
        excluded_variants[0].get("variant_id").and_then(serde_json::Value::as_str),
        Some("chr1:30:G:A")
    );
    assert_eq!(
        excluded_variants[0].get("missingness").and_then(serde_json::Value::as_f64),
        Some(2.0 / 3.0)
    );

    let maf_summary = payload.get("maf_summary").expect("maf_summary");
    assert_eq!(
        maf_summary.get("observed_variant_count").and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(
        maf_summary.get("allele_frequency_mean").and_then(serde_json::Value::as_f64),
        Some(0.2)
    );

    let heterozygosity = payload.get("heterozygosity").expect("heterozygosity");
    assert_eq!(
        heterozygosity.get("heterozygous_call_count").and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(
        heterozygosity.get("homozygous_alt_call_count").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(heterozygosity.get("het_hom_ratio").and_then(serde_json::Value::as_f64), Some(2.0));

    let repo_root = support::repo_root().expect("repo root");
    let qc_json_path = repo_root.join("runs/bench/local-smoke/vcf.qc/plink2/qc.json");
    let raw = std::fs::read_to_string(&qc_json_path).expect("read qc report");
    let qc_report: serde_json::Value = serde_json::from_str(&raw).expect("parse qc report");
    assert_eq!(
        qc_report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_qc_smoke.v1")
    );
    assert_eq!(qc_report.get("tool_id").and_then(serde_json::Value::as_str), Some("plink2"));
}
