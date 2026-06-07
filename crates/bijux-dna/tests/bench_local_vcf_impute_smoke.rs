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
fn bench_local_vcf_impute_smoke_reports_masked_truth_contract() {
    let payload = run_cli_json(&["bench", "local", "run-vcf-impute-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_impute_smoke.v1")
    );
    assert_eq!(
        payload.get("command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-impute-smoke --tool-id beagle")
    );
    assert_eq!(payload.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.impute"));
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("beagle"));
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(
        payload.get("input_fixture_id").and_then(serde_json::Value::as_str),
        Some("masked_truth_two_sample")
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
        Some("runs/bench/local-smoke/vcf.impute/beagle/artifacts/input/impute_input.vcf")
    );
    assert_eq!(
        payload.get("truth_vcf_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.impute/beagle/artifacts/input/impute_truth.vcf")
    );
    assert_eq!(
        payload.get("output_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.impute/beagle")
    );
    assert_eq!(
        payload.get("output_vcf_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.impute/beagle/imputed.vcf.gz")
    );
    assert_eq!(
        payload.get("output_tbi_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.impute/beagle/imputed.vcf.gz.tbi")
    );
    assert_eq!(
        payload.get("panel_assets_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.impute/beagle/panel_assets.json")
    );
    assert_eq!(
        payload.get("metrics_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.impute/beagle/metrics.json")
    );
    assert_eq!(
        payload.get("stage_result_manifest_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.impute/beagle/stage-result.json")
    );
    assert_eq!(payload.get("exit_code").and_then(serde_json::Value::as_i64), Some(0));
    assert_eq!(payload.get("variant_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("missing_before").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("missing_after").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("imputed_genotypes").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("low_confidence_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("masked_truth_site_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        payload.get("masked_truth_match_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(payload.get("unresolved_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        payload.get("sample_ids").and_then(serde_json::Value::as_array).map(|rows| {
            rows.iter()
                .map(|row| row.as_str().expect("sample id string").to_string())
                .collect::<Vec<_>>()
        }),
        Some(vec!["masked_sample".to_string(), "donor_sample".to_string()])
    );
    assert_eq!(payload.get("masked_sample_gt").and_then(serde_json::Value::as_str), Some("0/1"));
    assert_eq!(payload.get("donor_sample_gt").and_then(serde_json::Value::as_str), Some("0/1"));
    assert_eq!(
        payload
            .get("not_imputable_reasons")
            .and_then(serde_json::Value::as_object)
            .map(|rows| rows.len()),
        Some(0)
    );
    assert_eq!(payload.get("parseable").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("gt_present").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("gl_present").and_then(serde_json::Value::as_bool), Some(true));

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
    let metrics_path = repo_root.join("runs/bench/local-smoke/vcf.impute/beagle/metrics.json");
    let raw = std::fs::read_to_string(&metrics_path).expect("read metrics");
    let metrics: serde_json::Value = serde_json::from_str(&raw).expect("parse metrics");
    assert_eq!(
        metrics.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_impute_smoke.metrics.v1")
    );
    assert_eq!(metrics.get("missing_before").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(metrics.get("missing_after").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(metrics.get("imputed_genotypes").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(metrics.get("low_confidence_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(metrics.get("masked_truth_site_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        metrics.get("masked_truth_match_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
}
