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
fn bench_local_validate_vcf_no_empty_output_writes_governed_json_file() {
    let output = run_cli(&["bench", "local", "validate-vcf-no-empty-output"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "benchmarks/readiness/local-ready/vcf/no-empty-output-check.json"
    );

    let repo_root = support::repo_root().expect("repo root");
    let report_path =
        repo_root.join("benchmarks/readiness/local-ready/vcf/no-empty-output-check.json");
    let raw = std::fs::read_to_string(&report_path).expect("read report");
    let parsed: serde_json::Value = serde_json::from_str(&raw).expect("parse report");

    assert_eq!(
        parsed.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_no_empty_output_check.v1")
    );
    assert_eq!(parsed.get("checked_output_count").and_then(serde_json::Value::as_u64), Some(61));
    assert_eq!(parsed.get("valid").and_then(serde_json::Value::as_bool), Some(true));

    let rows = parsed.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert!(rows.iter().all(|row| {
        row.get("output_path")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value.starts_with("target/local-smoke/vcf/"))
            && row.get("bytes").and_then(serde_json::Value::as_u64).is_some_and(|bytes| bytes > 0)
            && row.get("status").and_then(serde_json::Value::as_str) == Some("non_empty")
    }));

    let chunks_json = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("vcf.prepare_reference_panel")
                && row.get("output_id").and_then(serde_json::Value::as_str) == Some("chunks_json")
        })
        .expect("chunks_json row");
    assert_eq!(chunks_json.get("output_kind").and_then(serde_json::Value::as_str), Some("json"));
    assert_eq!(
        chunks_json.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-smoke/vcf/vcf.prepare_reference_panel/bcftools/artifacts/chunks.json")
    );
}
