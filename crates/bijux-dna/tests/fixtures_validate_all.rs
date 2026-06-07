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
fn fixtures_validate_all_reports_benchmark_root_pass_state() {
    let payload = run_cli_json(&[
        "fixtures",
        "validate",
        "--root",
        "benchmarks/tests/fixtures",
        "--all",
        "--json",
    ]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.fixture_root_validation.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/benchmark-fixture-root-validation.json")
    );
    assert_eq!(
        payload.get("root_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures")
    );
    assert_eq!(payload.get("required_subroot_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("parser_domain_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("checked_fixture_count").and_then(serde_json::Value::as_u64), Some(17));
    assert_eq!(payload.get("invalid_fixture_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert!(rows.iter().any(|row| {
        row.get("fixture_kind").and_then(serde_json::Value::as_str) == Some("expected_truth")
            && row.get("fixture_id").and_then(serde_json::Value::as_str) == Some("vcf-mini")
            && row.get("detail_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/tests/fixtures/corpora/vcf-mini/expected")
    }));
    assert!(rows.iter().any(|row| {
        row.get("fixture_kind").and_then(serde_json::Value::as_str) == Some("database")
            && row.get("fixture_id").and_then(serde_json::Value::as_str) == Some("taxonomy-mini")
            && row.get("manifest_path").and_then(serde_json::Value::as_str)
                == Some("benchmarks/tests/fixtures/databases/taxonomy-mini/manifest.toml")
    }));
}
