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
fn bench_local_vcf_prepare_reference_panel_smoke_reports_real_governed_outputs() {
    let payload =
        run_cli_json(&["bench", "local", "run-vcf-prepare-reference-panel-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_prepare_reference_panel_smoke.v1")
    );
    assert_eq!(
        payload.get("command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-prepare-reference-panel-smoke --tool-id bcftools")
    );
    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.prepare_reference_panel")
    );
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("bcftools"));
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(
        payload.get("input_fixture_id").and_then(serde_json::Value::as_str),
        Some("reference_panel_sort_and_deduplicate")
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
        Some(
            "runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/artifacts/input/prepare_reference_panel_input.vcf",
        )
    );
    assert_eq!(
        payload.get("raw_panel_path").and_then(serde_json::Value::as_str),
        Some(
            "runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/artifacts/input/panel_store/hsapiens_grch38_mini/local-reference-panel/raw/panel.vcf.gz",
        )
    );
    assert_eq!(
        payload.get("output_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools")
    );
    assert_eq!(
        payload.get("panel_vcf_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/panel.vcf.gz")
    );
    assert_eq!(
        payload.get("panel_tbi_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/panel.vcf.gz.tbi")
    );
    assert_eq!(
        payload.get("metrics_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/metrics.json")
    );
    assert_eq!(
        payload.get("stage_result_manifest_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/stage-result.json")
    );
    assert_eq!(payload.get("exit_code").and_then(serde_json::Value::as_i64), Some(0));
    assert_eq!(payload.get("input_variants").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("output_variants").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        payload.get("sample_ids").and_then(serde_json::Value::as_array).map(|rows| {
            rows.iter()
                .map(|row| row.as_str().expect("sample id string").to_string())
                .collect::<Vec<_>>()
        }),
        Some(vec!["sample1".to_string()])
    );
    assert_eq!(payload.get("sample_consistent").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("duplicate_sites_removed").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        payload.get("normalization_status").and_then(serde_json::Value::as_str),
        Some("sorted_indexed_deduplicated")
    );
    assert_eq!(
        payload.get("index_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/panel.vcf.gz.tbi")
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
    let metrics_path =
        repo_root.join("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/metrics.json");
    let raw = std::fs::read_to_string(&metrics_path).expect("read metrics");
    let metrics: serde_json::Value = serde_json::from_str(&raw).expect("parse metrics");
    assert_eq!(
        metrics.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_prepare_reference_panel_smoke.metrics.v1")
    );
    assert_eq!(metrics.get("input_variants").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(metrics.get("output_variants").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(metrics.get("sample_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(metrics.get("duplicate_sites_removed").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        metrics.get("normalization_status").and_then(serde_json::Value::as_str),
        Some("sorted_indexed_deduplicated")
    );
}
