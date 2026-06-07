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
fn bench_readiness_vcf_parser_coverage_reports_benchmark_ready_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-vcf-parser-coverage", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_parser_coverage.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf-parser-coverage.tsv")
    );
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(payload.get("tool_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(payload.get("covered_row_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(payload.get("missing_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("parser_coverage_percent").and_then(serde_json::Value::as_f64),
        Some(100.0)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 9);

    assert!(rows.iter().all(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("coverage_status").and_then(serde_json::Value::as_str) == Some("covered")
            && row.get("fixture_path").and_then(serde_json::Value::as_str).is_some_and(|path| {
                path.starts_with("benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/")
            })
            && row
                .get("schema_id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|schema| schema.starts_with("bijux.vcf."))
    }));

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("parse_bcftools_call_metrics")
            && row.get("schema_id").and_then(serde_json::Value::as_str) == Some("bijux.vcf.call.v1")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.call_gl")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("parse_bcftools_call_gl_metrics")
            && row.get("schema_id").and_then(serde_json::Value::as_str)
                == Some("bijux.vcf.call_gl.v1")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.postprocess")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("parse_bcftools_postprocess_metrics")
            && row.get("schema_id").and_then(serde_json::Value::as_str)
                == Some("bijux.vcf.postprocess.v1")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.stats")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("parse_bcftools_stats_metrics")
            && row.get("schema_id").and_then(serde_json::Value::as_str)
                == Some("bijux.vcf.stats.v1")
    }));
}
