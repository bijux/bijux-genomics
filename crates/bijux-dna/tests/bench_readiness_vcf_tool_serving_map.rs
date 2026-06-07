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
fn bench_readiness_vcf_tool_serving_map_reports_owned_matrix_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-tool-serving-map", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_tool_serving_map.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("vcf"));
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf-tool-serving-map.tsv")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(9)
    );
    assert_eq!(
        payload.get("not_benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(11)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 20);

    let has_row = |tool_id: &str,
                   stage_id: &str,
                   support_status: &str,
                   adapter_status: &str,
                   parser_status: &str,
                   corpus_status: &str,
                   asset_status: &str,
                   benchmark_status: &str| {
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
                && row.get("support_status").and_then(serde_json::Value::as_str)
                    == Some(support_status)
                && row.get("adapter_status").and_then(serde_json::Value::as_str)
                    == Some(adapter_status)
                && row.get("parser_status").and_then(serde_json::Value::as_str)
                    == Some(parser_status)
                && row.get("corpus_status").and_then(serde_json::Value::as_str)
                    == Some(corpus_status)
                && row.get("asset_status").and_then(serde_json::Value::as_str) == Some(asset_status)
                && row.get("benchmark_status").and_then(serde_json::Value::as_str)
                    == Some(benchmark_status)
        })
    };

    assert!(
        has_row(
            "bcftools",
            "vcf.call",
            "supported",
            "runnable",
            "parse_normalized",
            "fixture:vcf_production_regression",
            "assigned",
            "benchmark_ready",
        ),
        "VCF tool-serving map must retain the governed calling row"
    );
    assert!(
        has_row(
            "bcftools",
            "vcf.call_gl",
            "supported",
            "runnable",
            "parse_normalized",
            "fixture:vcf_production_regression",
            "assigned",
            "benchmark_ready",
        ),
        "VCF tool-serving map must retain the governed GL calling row"
    );
    assert!(
        has_row(
            "bcftools",
            "vcf.postprocess",
            "supported",
            "runnable",
            "parse_normalized",
            "fixture:vcf_production_regression",
            "assigned",
            "benchmark_ready",
        ),
        "VCF tool-serving map must retain the governed postprocess row"
    );
    assert!(
        has_row(
            "bcftools",
            "vcf.prepare_reference_panel",
            "planned",
            "declared_only",
            "parse_normalized",
            "fixture:vcf_production_regression",
            "assigned",
            "not_benchmark_ready",
        ),
        "VCF tool-serving map must retain the governed reference-panel row"
    );
    assert!(
        has_row(
            "shapeit5",
            "vcf.phasing",
            "planned",
            "declared_only",
            "parse_normalized",
            "fixture:vcf_production_regression",
            "assigned",
            "not_benchmark_ready",
        ),
        "VCF tool-serving map must retain the governed phasing row"
    );
    assert!(
        has_row(
            "plink2",
            "vcf.population_structure",
            "planned",
            "declared_only",
            "parse_normalized",
            "fixture:vcf_production_regression",
            "assigned",
            "not_benchmark_ready",
        ),
        "VCF tool-serving map must retain the governed population-structure row"
    );
    assert!(
        has_row(
            "ibdne",
            "vcf.demography",
            "planned",
            "declared_only",
            "parse_normalized",
            "fixture:vcf_production_regression",
            "not_required",
            "not_benchmark_ready",
        ),
        "VCF tool-serving map must retain the governed demography row"
    );
}
