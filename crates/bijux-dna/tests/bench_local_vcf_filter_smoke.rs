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
fn bench_local_vcf_filter_smoke_reports_real_governed_outputs() {
    let payload = run_cli_json(&["bench", "local", "run-vcf-filter-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_filter_smoke.v1")
    );
    assert_eq!(
        payload.get("command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-filter-smoke --tool-id bcftools")
    );
    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.filter"));
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("bcftools"));
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(
        payload.get("input_fixture_id").and_then(serde_json::Value::as_str),
        Some("site_filter_single_sample")
    );
    assert_eq!(payload.get("sample_name").and_then(serde_json::Value::as_str), Some("sample_a"));
    assert_eq!(
        payload.get("input_vcf_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.filter/bcftools/artifacts/input/filter_input.vcf")
    );
    assert_eq!(
        payload.get("output_root").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.filter/bcftools")
    );
    assert_eq!(
        payload.get("output_vcf_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.filter/bcftools/filtered.vcf.gz")
    );
    assert_eq!(
        payload.get("output_tbi_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.filter/bcftools/filtered.vcf.gz.tbi")
    );
    assert_eq!(
        payload.get("metrics_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.filter/bcftools/metrics.json")
    );
    assert_eq!(
        payload.get("filter_breakdown_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.filter/bcftools/filter_breakdown.json")
    );
    assert_eq!(
        payload.get("filter_breakdown_tsv_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.filter/bcftools/filter_breakdown.tsv")
    );
    assert_eq!(
        payload.get("filter_explain_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.filter/bcftools/filter_explain.json")
    );
    assert_eq!(
        payload.get("stage_result_manifest_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.filter/bcftools/stage-result.json")
    );
    assert_eq!(payload.get("exit_code").and_then(serde_json::Value::as_i64), Some(0));
    assert_eq!(payload.get("input_variants").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("pass_variants").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("failed_variants").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("depth_threshold").and_then(serde_json::Value::as_f64), Some(8.0));
    assert_eq!(payload.get("quality_threshold").and_then(serde_json::Value::as_f64), Some(30.0));
    assert_eq!(payload.get("missingness_threshold").and_then(serde_json::Value::as_f64), Some(0.2));
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("parseable").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("gt_present").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("gl_present").and_then(serde_json::Value::as_bool), Some(false));
    assert_eq!(
        payload.get("filter_ids").and_then(serde_json::Value::as_array).map(|rows| {
            rows.iter()
                .map(|row| row.as_str().expect("filter id string").to_string())
                .collect::<Vec<_>>()
        }),
        Some(vec![
            "HIGH_MISSING".to_string(),
            "LOWQUAL".to_string(),
            "LOW_DP".to_string(),
            "LOW_MQ".to_string(),
        ])
    );

    let checks = payload
        .get("validation_checks")
        .and_then(serde_json::Value::as_object)
        .expect("validation_checks object");
    assert_eq!(checks.get("bgzip").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(checks.get("tabix_index").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(checks.get("sorted").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(checks.get("contig_header_sane").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(checks.get("sample_ids_valid").and_then(serde_json::Value::as_bool), Some(true));

    let repo_root = support::repo_root().expect("repo root");
    let metrics_path = repo_root.join("target/local-smoke/vcf.filter/bcftools/metrics.json");
    let raw = std::fs::read_to_string(&metrics_path).expect("read metrics");
    let metrics: serde_json::Value = serde_json::from_str(&raw).expect("parse metrics");
    assert_eq!(
        metrics.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_filter_smoke.metrics.v1")
    );
    assert_eq!(metrics.get("input_variants").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(metrics.get("pass_variants").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(metrics.get("failed_variants").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(metrics.get("depth_threshold").and_then(serde_json::Value::as_f64), Some(8.0));
    assert_eq!(metrics.get("quality_threshold").and_then(serde_json::Value::as_f64), Some(30.0));
    assert_eq!(metrics.get("missingness_threshold").and_then(serde_json::Value::as_f64), Some(0.2));
}
