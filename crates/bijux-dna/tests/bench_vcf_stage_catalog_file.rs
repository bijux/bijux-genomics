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
fn bench_vcf_stage_catalog_writes_governed_toml_file() {
    let output = run_cli(&["bench", "local", "render-vcf-stage-catalog"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "configs/bench/local/vcf-stage-catalog.toml"
    );

    let repo_root = support::repo_root().expect("repo root");
    let config_path = repo_root.join("configs/bench/local/vcf-stage-catalog.toml");
    let raw = std::fs::read_to_string(&config_path).expect("read config");
    let parsed: toml::Value = toml::from_str(&raw).expect("parse config");

    assert_eq!(
        parsed.get("schema_version").and_then(toml::Value::as_str),
        Some("bijux.bench.vcf.local_stage_catalog.v1")
    );

    let rows = parsed.get("rows").and_then(toml::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 20);
    assert!(rows.iter().all(|row| {
        row.get("stage_id")
            .and_then(toml::Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
            && row
                .get("stage_name")
                .and_then(toml::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
            && row
                .get("input_types")
                .and_then(toml::Value::as_array)
                .is_some_and(|values| !values.is_empty())
            && row
                .get("output_types")
                .and_then(toml::Value::as_array)
                .is_some_and(|values| !values.is_empty())
            && row.get("required_assets").and_then(toml::Value::as_array).is_some()
            && row
                .get("benchmark_category")
                .and_then(toml::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
            && row
                .get("local_smoke_mode")
                .and_then(toml::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
    }));

    let phasing = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(toml::Value::as_str) == Some("vcf.phasing"))
        .expect("phasing row");
    assert_eq!(phasing.get("default_tool_id").and_then(toml::Value::as_str), Some("shapeit5"));
    assert_eq!(
        phasing.get("local_smoke_mode").and_then(toml::Value::as_str),
        Some("vcf_cohort_with_panel")
    );
    assert!(phasing.get("required_assets").and_then(toml::Value::as_array).is_some_and(|assets| {
        assets.iter().any(|value| value.as_str() == Some("genetic_map"))
            && assets.iter().any(|value| value.as_str() == Some("reference_panel_lock"))
    }));

    let stats = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(toml::Value::as_str) == Some("vcf.stats"))
        .expect("stats row");
    assert_eq!(stats.get("support_status").and_then(toml::Value::as_str), Some("supported"));
    assert_eq!(
        stats.get("metrics_schema_id").and_then(toml::Value::as_str),
        Some("bijux.vcf.stats.v1")
    );
    assert_eq!(
        stats.get("benchmark_category").and_then(toml::Value::as_str),
        Some("quality_control")
    );
}
