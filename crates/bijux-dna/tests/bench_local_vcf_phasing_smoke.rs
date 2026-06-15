#![allow(clippy::expect_used, clippy::too_many_lines)]

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
fn bench_local_vcf_phasing_smoke_reports_real_governed_outputs() {
    let payload = run_cli_json(&["bench", "local", "run-vcf-phasing-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_phasing_smoke.v1")
    );
    assert_eq!(
        payload.get("command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-phasing-smoke --tool-id shapeit5")
    );
    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.phasing"));
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("shapeit5"));
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(
        payload.get("input_fixture_id").and_then(serde_json::Value::as_str),
        Some("cohort_unphased_two_sample")
    );
    assert_eq!(
        payload.get("panel_id").and_then(serde_json::Value::as_str),
        Some("hsapiens_grch38_mini")
    );
    assert_eq!(
        payload.get("map_id").and_then(serde_json::Value::as_str),
        Some("hsapiens_grch38_chr_map")
    );
    assert_eq!(
        payload.get("input_vcf_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.phasing/shapeit5/artifacts/input/phasing_input.vcf")
    );
    assert_eq!(
        payload.get("output_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.phasing/shapeit5")
    );
    assert_eq!(
        payload.get("output_vcf_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.phasing/shapeit5/phased.vcf.gz")
    );
    assert_eq!(
        payload.get("output_tbi_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.phasing/shapeit5/phased.vcf.gz.tbi")
    );
    assert_eq!(
        payload.get("panel_assets_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.phasing/shapeit5/panel_assets.json")
    );
    assert_eq!(
        payload.get("metrics_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.phasing/shapeit5/metrics.json")
    );
    assert_eq!(
        payload.get("stage_result_manifest_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.phasing/shapeit5/stage-result.json")
    );
    assert_eq!(payload.get("exit_code").and_then(serde_json::Value::as_i64), Some(0));
    assert_eq!(payload.get("input_genotypes").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("phased_genotypes").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("unphased_genotypes").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("phase_set_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        payload.get("sample_ids").and_then(serde_json::Value::as_array).map(|rows| {
            rows.iter()
                .map(|row| row.as_str().expect("sample id string").to_string())
                .collect::<Vec<_>>()
        }),
        Some(vec!["cohort_alpha".to_string(), "cohort_beta".to_string()])
    );
    assert_eq!(payload.get("parseable").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("gt_present").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("gl_present").and_then(serde_json::Value::as_bool), Some(false));

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
    let metrics_path = repo_root.join("runs/bench/local-smoke/vcf.phasing/shapeit5/metrics.json");
    let raw = std::fs::read_to_string(&metrics_path).expect("read metrics");
    let metrics: serde_json::Value = serde_json::from_str(&raw).expect("parse metrics");
    assert_eq!(
        metrics.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_phasing_smoke.metrics.v1")
    );
    assert_eq!(metrics.get("input_genotypes").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(metrics.get("phased_genotypes").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(metrics.get("unphased_genotypes").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(metrics.get("phase_set_count").and_then(serde_json::Value::as_u64), Some(2));
}
