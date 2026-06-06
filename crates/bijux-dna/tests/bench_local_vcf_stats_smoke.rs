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
fn bench_local_vcf_stats_smoke_reports_normalized_governed_metrics() {
    let payload = run_cli_json(&["bench", "local", "run-vcf-stats-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_stats_smoke.v1")
    );
    assert_eq!(
        payload.get("command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-stats-smoke --tool-id bcftools")
    );
    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.stats"));
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("bcftools"));
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(
        payload.get("input_fixture_id").and_then(serde_json::Value::as_str),
        Some("stats_cohort_minimal")
    );
    assert_eq!(
        payload.get("sample_name").and_then(serde_json::Value::as_str),
        Some("cohort_stats")
    );
    assert_eq!(
        payload.get("input_vcf_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.stats/bcftools/artifacts/input/stats_input.vcf")
    );
    assert_eq!(
        payload.get("output_root").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.stats/bcftools")
    );
    assert_eq!(
        payload.get("stats_json_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.stats/bcftools/stats.json")
    );
    assert_eq!(
        payload.get("bcftools_stats_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.stats/bcftools/bcftools_stats.txt")
    );
    assert_eq!(
        payload.get("metrics_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.stats/bcftools/metrics.json")
    );
    assert_eq!(
        payload.get("stage_result_manifest_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf.stats/bcftools/stage-result.json")
    );
    assert_eq!(payload.get("exit_code").and_then(serde_json::Value::as_i64), Some(0));
    assert_eq!(payload.get("variant_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("snp_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("indel_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("transition_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("transversion_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("ti_tv").and_then(serde_json::Value::as_f64), Some(2.0));
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(2));

    let repo_root = support::repo_root().expect("repo root");
    let stats_json_path = repo_root.join("target/local-smoke/vcf.stats/bcftools/stats.json");
    let raw = std::fs::read_to_string(&stats_json_path).expect("read stats json");
    let stats: serde_json::Value = serde_json::from_str(&raw).expect("parse stats json");
    assert_eq!(
        stats.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.vcf.stats.v1")
    );
    assert_eq!(stats.get("sample_name").and_then(serde_json::Value::as_str), Some("cohort_stats"));
    assert_eq!(stats.get("variants_total").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(stats.get("sample_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(stats.get("snps").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(stats.get("indels").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(stats.get("ti_tv").and_then(serde_json::Value::as_f64), Some(2.0));
}
