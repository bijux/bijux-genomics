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
fn bench_readiness_vcf_undercovered_stages_reports_governed_stage_slice() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-undercovered-stages", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_undercovered_stages.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("vcf"));
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf-undercovered-stages.tsv")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(
        payload.get("undercovered_stage_count").and_then(serde_json::Value::as_u64),
        Some(10)
    );
    assert_eq!(
        payload
            .get("decision_counts")
            .and_then(|value| value.get("future_not_benchmark_ready"))
            .and_then(serde_json::Value::as_u64),
        Some(9)
    );
    assert_eq!(
        payload
            .get("decision_counts")
            .and_then(|value| value.get("limit_to_specialized_tool"))
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 10);

    let has_row = |stage_id: &str,
                   valid_tool_classes: &[&str],
                   registered_tools: &[&str],
                   missing_tools: &[&str],
                   decision: &str| {
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
                && row.get("valid_tool_classes").and_then(serde_json::Value::as_array).map(
                    |values| {
                        values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()
                    },
                ) == Some(valid_tool_classes.to_vec())
                && row.get("registered_tools").and_then(serde_json::Value::as_array).map(|values| {
                    values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()
                }) == Some(registered_tools.to_vec())
                && row.get("missing_tools").and_then(serde_json::Value::as_array).map(|values| {
                    values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()
                }) == Some(missing_tools.to_vec())
                && row.get("decision").and_then(serde_json::Value::as_str) == Some(decision)
        })
    };

    assert!(has_row(
        "vcf.admixture",
        &["cohort_analysis"],
        &["plink2"],
        &["plink"],
        "future_not_benchmark_ready",
    ));
    assert!(has_row(
        "vcf.phasing",
        &["phasing"],
        &["shapeit5"],
        &["beagle", "eagle", "shapeit"],
        "future_not_benchmark_ready",
    ));
    assert!(has_row(
        "vcf.ibd",
        &["demography", "relatedness"],
        &["germline"],
        &["ibdhap", "ibdne", "ibdseq"],
        "future_not_benchmark_ready",
    ));
    assert!(has_row(
        "vcf.impute",
        &["imputation", "phasing"],
        &["beagle"],
        &["glimpse", "impute5", "minimac4"],
        "future_not_benchmark_ready",
    ));
    assert!(has_row(
        "vcf.imputation_metrics",
        &["imputation", "phasing"],
        &["beagle"],
        &["glimpse", "impute5", "minimac4"],
        "future_not_benchmark_ready",
    ));
    assert!(has_row(
        "vcf.population_structure",
        &["cohort_analysis", "population_structure"],
        &["plink2"],
        &["eigensoft", "plink"],
        "limit_to_specialized_tool",
    ));
    assert!(!rows
        .iter()
        .any(|row| { row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.pca") }));
    assert!(!rows
        .iter()
        .any(|row| { row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.qc") }));
}
