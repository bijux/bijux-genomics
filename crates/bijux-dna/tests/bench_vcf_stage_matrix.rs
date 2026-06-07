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
fn bench_vcf_stage_matrix_matches_owned_vcf_contracts() {
    let payload = run_cli_json(&["bench", "local", "render-vcf-stage-matrix", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_stage_matrix.v1")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/configs/local/vcf-stage-matrix.toml")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("supported_stage_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(payload.get("planned_stage_count").and_then(serde_json::Value::as_u64), Some(11));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 20);

    let prepare_reference_panel = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.prepare_reference_panel")
        })
        .expect("prepare reference panel row");
    assert_eq!(
        prepare_reference_panel.get("tool_id").and_then(serde_json::Value::as_str),
        Some("bcftools")
    );
    assert_eq!(
        prepare_reference_panel.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("vcf_production_regression")
    );
    assert_eq!(
        prepare_reference_panel.get("asset_profile_id").and_then(serde_json::Value::as_str),
        Some("vcf_reference_panel")
    );
    assert_eq!(
        prepare_reference_panel.get("adapter_id").and_then(serde_json::Value::as_str),
        Some("vcf.adapter.reference_panel")
    );
    assert_eq!(
        prepare_reference_panel.get("parser_id").and_then(serde_json::Value::as_str),
        Some("vcf.parser.vcf_output")
    );
    assert_eq!(
        prepare_reference_panel
            .get("expected_outputs")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["prepared_panel", "chunks_json"])
    );

    let phasing = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.phasing"))
        .expect("phasing row");
    assert_eq!(phasing.get("tool_id").and_then(serde_json::Value::as_str), Some("shapeit5"));
    assert_eq!(
        phasing.get("asset_profile_id").and_then(serde_json::Value::as_str),
        Some("vcf_cohort_with_panel")
    );
    assert_eq!(
        phasing.get("adapter_id").and_then(serde_json::Value::as_str),
        Some("vcf.adapter.panel_workflow")
    );
    assert_eq!(
        phasing.get("parser_id").and_then(serde_json::Value::as_str),
        Some("vcf.parser.vcf_output")
    );
    assert_eq!(
        phasing
            .get("expected_outputs")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["phased_vcf"])
    );

    let imputation_metrics = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.imputation_metrics")
        })
        .expect("imputation metrics row");
    assert_eq!(
        imputation_metrics.get("tool_id").and_then(serde_json::Value::as_str),
        Some("beagle")
    );
    assert_eq!(
        imputation_metrics.get("adapter_id").and_then(serde_json::Value::as_str),
        Some("vcf.adapter.panel_workflow")
    );
    assert_eq!(
        imputation_metrics.get("parser_id").and_then(serde_json::Value::as_str),
        Some("vcf.parser.report_json")
    );
    assert_eq!(
        imputation_metrics
            .get("expected_outputs")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["imputation_metrics_json"])
    );

    let stats = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.stats"))
        .expect("stats row");
    assert_eq!(stats.get("tool_id").and_then(serde_json::Value::as_str), Some("bcftools"));
    assert_eq!(
        stats.get("adapter_id").and_then(serde_json::Value::as_str),
        Some("vcf.adapter.quality_control")
    );
    assert_eq!(
        stats.get("parser_id").and_then(serde_json::Value::as_str),
        Some("vcf.parser.stats_report")
    );
    assert_eq!(
        stats
            .get("expected_outputs")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["stats_json"])
    );
}
