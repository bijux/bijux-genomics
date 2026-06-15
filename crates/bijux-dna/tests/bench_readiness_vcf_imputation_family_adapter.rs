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
fn bench_readiness_vcf_imputation_family_adapter_reports_governed_rows() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-vcf-imputation-family-adapter", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_imputation_family_adapter.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("vcf"));
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/adapters/imputation-family.vcf.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(payload.get("parser_output_row_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(
        payload.get("missing_input_test_passed_row_count").and_then(serde_json::Value::as_u64),
        Some(8)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 8);

    for (tool_id, stage_id, benchmark_status) in [
        ("beagle", "vcf.imputation_metrics", "benchmark_ready"),
        ("beagle", "vcf.impute", "benchmark_ready"),
        ("glimpse", "vcf.imputation_metrics", "not_benchmark_ready"),
        ("glimpse", "vcf.impute", "not_benchmark_ready"),
        ("impute5", "vcf.imputation_metrics", "not_benchmark_ready"),
        ("impute5", "vcf.impute", "not_benchmark_ready"),
        ("minimac4", "vcf.imputation_metrics", "not_benchmark_ready"),
        ("minimac4", "vcf.impute", "not_benchmark_ready"),
    ] {
        assert!(
            rows.iter().any(|row| {
                row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
                    && row.get("benchmark_status").and_then(serde_json::Value::as_str)
                        == Some(benchmark_status)
                    && row.get("argv_validation_passed").and_then(serde_json::Value::as_bool)
                        == Some(true)
                    && row.get("missing_input_test_passed").and_then(serde_json::Value::as_bool)
                        == Some(true)
            }),
            "report must retain governed imputation row {tool_id} / {stage_id}"
        );
    }

    let beagle_imputation = rows
        .iter()
        .find(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("vcf.imputation_metrics")
        })
        .expect("beagle imputation row");
    assert!(
        beagle_imputation
            .get("target_vcf_path")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|path| {
                path.ends_with(
                    "benchmarks/readiness/adapters/imputation/beagle/vcf.imputation_metrics/artifacts/input/vcf_imputation_metrics.vcf.gz",
                )
            }),
        "beagle imputation row must retain the materialized indexed target VCF path"
    );
    assert_eq!(
        beagle_imputation.get("parser_id").and_then(serde_json::Value::as_str),
        Some("vcf.parser.report_json")
    );
    assert_eq!(
        beagle_imputation.get("quality_output_path").and_then(serde_json::Value::as_str),
        Some(
            "benchmarks/readiness/adapters/imputation/beagle/vcf.imputation_metrics/imputation_metrics.json"
        )
    );
    assert_eq!(
        beagle_imputation
            .get("parser_output_ids")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(5)
    );
    assert!(
        beagle_imputation
            .get("declared_outputs")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| {
                items.iter().any(|item| {
                    item.get("artifact_id").and_then(serde_json::Value::as_str)
                        == Some("imputation_metrics_json")
                }) && items.iter().any(|item| {
                    item.get("artifact_id").and_then(serde_json::Value::as_str)
                        == Some("orchestration_manifest_json")
                })
            }),
        "vcf.imputation_metrics rows must declare metrics and orchestration artifacts"
    );

    let glimpse_impute = rows
        .iter()
        .find(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("glimpse")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.impute")
        })
        .expect("glimpse impute row");
    assert_eq!(
        glimpse_impute.get("region_literal").and_then(serde_json::Value::as_str),
        Some("1:1-1000000")
    );
    let glimpse_argv = glimpse_impute
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("argv"))
        .and_then(serde_json::Value::as_array)
        .expect("glimpse argv");
    let glimpse_joined =
        glimpse_argv.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>().join(" ");
    for needle in ["GLIMPSE_phase", "--input-region", "--output-region"] {
        assert!(glimpse_joined.contains(needle), "glimpse row must retain {needle}");
    }

    let minimac_impute = rows
        .iter()
        .find(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("minimac4")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.impute")
        })
        .expect("minimac4 impute row");
    assert!(
        minimac_impute
            .get("panel_m3vcf_path")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|path| path.ends_with("panel.m3vcf.gz")),
        "minimac4 row must retain the governed m3vcf panel path"
    );
    let minimac_argv = minimac_impute
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.first())
        .and_then(|step| step.get("argv"))
        .and_then(serde_json::Value::as_array)
        .expect("minimac4 argv");
    let minimac_joined =
        minimac_argv.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>().join(" ");
    for needle in ["minimac4", "--refHaps", "--prefix"] {
        assert!(minimac_joined.contains(needle), "minimac4 row must retain {needle}");
    }
}
