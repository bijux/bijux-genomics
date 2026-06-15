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
fn bench_validate_matrix_vcf_strict_accepts_governed_matrix() {
    let payload =
        run_cli_json(&["bench", "validate-matrix", "--domain", "vcf", "--strict", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.validate_matrix.v1")
    );
    assert_eq!(payload.get("domain").and_then(serde_json::Value::as_str), Some("vcf"));
    assert_eq!(
        payload.get("matrix_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/configs/local/vcf-stage-matrix.toml")
    );
    assert_eq!(payload.get("strict").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("stage_count").and_then(serde_json::Value::as_u64), Some(20));
    assert!(
        payload.get("required_tool_count").and_then(serde_json::Value::as_u64).unwrap_or(0) >= 8
    );
    assert!(
        payload.get("registry_tool_count").and_then(serde_json::Value::as_u64).unwrap_or(0) >= 8
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 20);
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.phasing")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("shapeit5")
            && row.get("adapter_id").and_then(serde_json::Value::as_str)
                == Some("vcf.adapter.panel_workflow")
            && row.get("parser_id").and_then(serde_json::Value::as_str)
                == Some("vcf.parser.vcf_output")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.stats")
            && row.get("expected_outputs").and_then(serde_json::Value::as_array).is_some_and(
                |outputs| outputs.len() == 1 && outputs[0].as_str() == Some("stats_json"),
            )
    }));
}
