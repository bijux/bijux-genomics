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
fn bench_validate_vcf_schemas_reports_governed_pass_state() {
    let payload = run_cli_json(&["bench", "validate-schemas", "--domain", "vcf", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.vcf_schema_validation.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("vcf"));
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/vcf-schema-validation.json")
    );
    assert_eq!(
        payload.get("shared_schema_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/schemas/vcf-normalized-metrics.v1.json")
    );
    assert_eq!(
        payload.get("stage_dir").and_then(serde_json::Value::as_str),
        Some("benchmarks/schemas/vcf-normalized-metrics")
    );
    assert_eq!(payload.get("passes_gate"), Some(&serde_json::Value::Bool(true)));
    assert_eq!(payload.get("shared_schema_matches"), Some(&serde_json::Value::Bool(true)));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("required_stage_count").and_then(serde_json::Value::as_u64), Some(17));
    assert_eq!(
        payload.get("exact_stage_schema_file_count").and_then(serde_json::Value::as_u64),
        Some(20)
    );
    assert_eq!(
        payload
            .get("missing_stage_schema_files")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(0)
    );
    assert_eq!(
        payload
            .get("unexpected_stage_schema_files")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(0)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 20);
    assert!(rows.iter().all(|row| {
        row.get("file_present") == Some(&serde_json::Value::Bool(true))
            && row.get("exact_match") == Some(&serde_json::Value::Bool(true))
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.pca")
            && row.get("schema_version").and_then(serde_json::Value::as_str)
                == Some("bijux.vcf.pca.v1")
    }));
}
