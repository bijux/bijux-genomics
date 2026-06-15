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
fn bench_readiness_vcf_comparable_metrics_reports_governed_metric_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-comparable-metrics", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_comparable_metrics.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf-comparable-metrics.tsv")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(payload.get("multi_tool_stage_count").and_then(serde_json::Value::as_u64), Some(12));
    assert_eq!(
        payload.get("retained_tool_row_count").and_then(serde_json::Value::as_u64),
        Some(31)
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(35));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 35);

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call_gl")
            && row.get("metric_id").and_then(serde_json::Value::as_str)
                == Some("sites_with_likelihoods")
            && row.get("metric_name").and_then(serde_json::Value::as_str)
                == Some("sites with likelihoods")
            && row.get("unit").and_then(serde_json::Value::as_str) == Some("sites")
            && row.get("direction").and_then(serde_json::Value::as_str) == Some("higher_is_better")
            && row.get("required").and_then(serde_json::Value::as_bool) == Some(true)
            && row.get("tools_covered").and_then(serde_json::Value::as_array).is_some_and(|tools| {
                tools
                    == &[
                        serde_json::Value::String("angsd".to_string()),
                        serde_json::Value::String("bcftools".to_string()),
                    ]
            })
    }));

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.qc")
            && row.get("metric_id").and_then(serde_json::Value::as_str) == Some("concordance")
            && row.get("unit").and_then(serde_json::Value::as_str) == Some("fraction")
            && row.get("direction").and_then(serde_json::Value::as_str) == Some("higher_is_better")
            && row.get("tools_covered").and_then(serde_json::Value::as_array).is_some_and(|tools| {
                tools
                    == &[
                        serde_json::Value::String("bcftools".to_string()),
                        serde_json::Value::String("plink".to_string()),
                        serde_json::Value::String("plink2".to_string()),
                    ]
            })
    }));

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.phasing")
            && row.get("metric_id").and_then(serde_json::Value::as_str)
                == Some("switch_error_proxy")
            && row.get("direction").and_then(serde_json::Value::as_str) == Some("lower_is_better")
            && row.get("tools_covered").and_then(serde_json::Value::as_array).is_some_and(|tools| {
                tools
                    == &[
                        serde_json::Value::String("beagle".to_string()),
                        serde_json::Value::String("eagle".to_string()),
                        serde_json::Value::String("shapeit5".to_string()),
                    ]
            })
    }));

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.impute")
            && row.get("metric_id").and_then(serde_json::Value::as_str)
                == Some("masked_truth_match_count")
            && row.get("direction").and_then(serde_json::Value::as_str) == Some("higher_is_better")
            && row.get("tools_covered").and_then(serde_json::Value::as_array).is_some_and(|tools| {
                tools
                    == &[
                        serde_json::Value::String("beagle".to_string()),
                        serde_json::Value::String("glimpse".to_string()),
                        serde_json::Value::String("impute5".to_string()),
                        serde_json::Value::String("minimac4".to_string()),
                    ]
            })
    }));

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.imputation_metrics")
            && row.get("metric_id").and_then(serde_json::Value::as_str) == Some("concordance")
            && row.get("unit").and_then(serde_json::Value::as_str) == Some("fraction")
            && row.get("direction").and_then(serde_json::Value::as_str) == Some("higher_is_better")
            && row.get("required").and_then(serde_json::Value::as_bool) == Some(true)
    }));

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.imputation_metrics")
            && row.get("metric_id").and_then(serde_json::Value::as_str) == Some("dosage_r2")
            && row.get("unit").and_then(serde_json::Value::as_str) == Some("score")
            && row.get("direction").and_then(serde_json::Value::as_str) == Some("higher_is_better")
            && row.get("required").and_then(serde_json::Value::as_bool) == Some(false)
    }));
}
