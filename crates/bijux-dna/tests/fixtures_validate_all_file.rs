#![allow(clippy::expect_used)]

use std::fs;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> (support::RepoSandbox, std::process::Output) {
    let sandbox = support::RepoSandbox::new("fixture-root-validation-").expect("repo sandbox");
    let home = tempfile::tempdir().expect("tempdir");

    let output = sandbox
        .bijux_dna_command()
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli");

    (sandbox, output)
}

#[test]
fn fixtures_validate_all_writes_benchmark_root_report_file() {
    let (sandbox, output) =
        run_cli(&["fixtures", "validate", "--root", "benchmarks/tests/fixtures", "--all"]);

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "benchmarks/readiness/benchmark-fixture-root-validation.json"
    );

    let report_path =
        sandbox.path().join("benchmarks/readiness/benchmark-fixture-root-validation.json");
    let report_raw = fs::read_to_string(&report_path).expect("read report");
    let report: serde_json::Value = serde_json::from_str(&report_raw).expect("parse report");

    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.fixture_root_validation.v1")
    );
    assert_eq!(
        report.get("root_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures")
    );
    assert_eq!(report.get("ok").and_then(serde_json::Value::as_bool), Some(true));
}
