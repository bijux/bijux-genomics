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
fn bench_readiness_vcf_bcftools_adapter_reports_governed_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-bcftools-adapter", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_bcftools_adapter.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("vcf"));
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("bcftools"));
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/adapters/bcftools.vcf.json")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(payload.get("supported_row_count").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(payload.get("planned_row_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(
        payload.get("argv_valid_row_count").and_then(serde_json::Value::as_u64),
        Some(10)
    );
    assert_eq!(
        payload.get("missing_input_test_passed_row_count")
            .and_then(serde_json::Value::as_u64),
        Some(10)
    );
    assert_eq!(payload.get("indexed_row_count").and_then(serde_json::Value::as_u64), Some(9));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 10);

    let has_stage = |stage_id: &str, support_status: &str, benchmark_status: &str| {
        rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some(support_status)
                && row.get("benchmark_status").and_then(serde_json::Value::as_str)
                    == Some(benchmark_status)
                && row.get("argv_validation_passed").and_then(serde_json::Value::as_bool)
                    == Some(true)
                && row.get("missing_input_test_passed").and_then(serde_json::Value::as_bool)
                    == Some(true)
        })
    };

    assert!(
        has_stage("vcf.call", "supported", "benchmark_ready"),
        "report must retain the governed bcftools calling row"
    );
    assert!(
        has_stage("vcf.call_diploid", "supported", "benchmark_ready"),
        "report must retain the governed diploid calling row"
    );
    assert!(
        has_stage("vcf.filter", "supported", "benchmark_ready"),
        "report must retain the governed filter row"
    );
    assert!(
        has_stage("vcf.stats", "supported", "benchmark_ready"),
        "report must retain the governed stats row"
    );
    assert!(
        has_stage("vcf.prepare_reference_panel", "planned", "not_benchmark_ready"),
        "report must retain the governed panel-preparation row"
    );
    assert!(
        has_stage("vcf.postprocess", "planned", "not_benchmark_ready"),
        "report must retain the governed postprocess row"
    );

    let call_row = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call"))
        .expect("call row");
    let call_steps = call_row
        .get("command_steps")
        .and_then(serde_json::Value::as_array)
        .expect("call steps");
    assert_eq!(call_steps.len(), 3, "call row must keep mpileup, call, and index steps");
    assert_eq!(
        call_steps[0].get("argv").and_then(serde_json::Value::as_array)
            .and_then(|argv| argv.first())
            .and_then(serde_json::Value::as_str),
        Some("bcftools")
    );

    let stats_row = rows
        .iter()
        .find(|row| row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.stats"))
        .expect("stats row");
    assert_eq!(
        stats_row.get("raw_output_ids").and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::Value::String("bcftools_stats_txt".to_string())])
    );
    assert_eq!(
        stats_row.get("parser_output_ids").and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::Value::String("stats_json".to_string())])
    );
}
