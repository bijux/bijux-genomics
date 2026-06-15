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
fn bench_vcf_stage_matrix_writes_governed_toml_file() {
    let output = run_cli(&["bench", "local", "render-vcf-stage-matrix"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "benchmarks/configs/local/vcf-stage-matrix.toml"
    );

    let repo_root = support::repo_root().expect("repo root");
    let config_path = repo_root.join("benchmarks/configs/local/vcf-stage-matrix.toml");
    let raw = std::fs::read_to_string(&config_path).expect("read config");
    let parsed: toml::Value = toml::from_str(&raw).expect("parse config");

    assert_eq!(
        parsed.get("schema_version").and_then(toml::Value::as_str),
        Some("bijux.bench.vcf.local_stage_matrix.v1")
    );

    let rows = parsed.get("rows").and_then(toml::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 20);
    assert!(rows.iter().all(|row| {
        row.get("stage_id")
            .and_then(toml::Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
            && row
                .get("tool_id")
                .and_then(toml::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
            && row
                .get("corpus_id")
                .and_then(toml::Value::as_str)
                .is_some_and(|value| value == "vcf_production_regression")
            && row
                .get("asset_profile_id")
                .and_then(toml::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
            && row
                .get("adapter_id")
                .and_then(toml::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
            && row
                .get("parser_id")
                .and_then(toml::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
            && row
                .get("expected_outputs")
                .and_then(toml::Value::as_array)
                .is_some_and(|values| !values.is_empty())
    }));

    let prepare_reference_panel = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("vcf.prepare_reference_panel")
        })
        .expect("prepare reference panel row");
    assert_eq!(
        prepare_reference_panel.get("adapter_id").and_then(toml::Value::as_str),
        Some("vcf.adapter.reference_panel")
    );
    assert_eq!(
        prepare_reference_panel.get("parser_id").and_then(toml::Value::as_str),
        Some("vcf.parser.vcf_output")
    );
    assert_eq!(
        prepare_reference_panel
            .get("expected_outputs")
            .and_then(toml::Value::as_array)
            .map(|values| values.iter().filter_map(toml::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["prepared_panel", "chunks_json"])
    );

    let imputation_metrics = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("vcf.imputation_metrics")
        })
        .expect("imputation metrics row");
    assert_eq!(
        imputation_metrics.get("adapter_id").and_then(toml::Value::as_str),
        Some("vcf.adapter.panel_workflow")
    );
    assert_eq!(
        imputation_metrics.get("parser_id").and_then(toml::Value::as_str),
        Some("vcf.parser.report_json")
    );
    assert_eq!(
        imputation_metrics
            .get("expected_outputs")
            .and_then(toml::Value::as_array)
            .map(|values| values.iter().filter_map(toml::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["imputation_metrics_json"])
    );

    let stats = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(toml::Value::as_str) == Some("vcf.stats"))
        .expect("stats row");
    assert_eq!(
        stats.get("adapter_id").and_then(toml::Value::as_str),
        Some("vcf.adapter.quality_control")
    );
    assert_eq!(
        stats.get("parser_id").and_then(toml::Value::as_str),
        Some("vcf.parser.stats_report")
    );
    assert_eq!(
        stats
            .get("expected_outputs")
            .and_then(toml::Value::as_array)
            .map(|values| values.iter().filter_map(toml::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["stats_json"])
    );
}
