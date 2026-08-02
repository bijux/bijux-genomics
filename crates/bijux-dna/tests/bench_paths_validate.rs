#![allow(clippy::expect_used, clippy::too_many_lines)]

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(sandbox: &support::RepoSandbox, args: &[&str]) -> serde_json::Value {
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
fn bench_paths_validate_reports_tracked_benchmark_roots() {
    let sandbox = support::RepoSandbox::new("benchmark-path-validation-").expect("repo sandbox");
    let _ = run_cli_json(
        &sandbox,
        &[
            "bench",
            "validate-schemas",
            "--schema-root",
            "benchmarks/schemas",
            "--domain",
            "fastq,bam,vcf",
            "--json",
        ],
    );
    let _ = run_cli_json(
        &sandbox,
        &["bench", "readiness", "render-all-domain-stage-tool-table", "--json"],
    );
    let payload = run_cli_json(&sandbox, &["bench", "paths", "validate", "--strict", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.paths_validate.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/benchmark-paths-validation.json")
    );
    assert_eq!(payload.get("strict").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("root_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(payload.get("existing_root_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(payload.get("tracked_marker_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(payload.get("ignored_root_count").and_then(serde_json::Value::as_u64), Some(0));
    assert!(payload
        .get("readiness_json_snapshot_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count >= 1));
    assert!(payload
        .get("readiness_tsv_snapshot_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count >= 1));
    assert!(payload
        .get("readiness_snapshot_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count >= 2));
    assert_eq!(
        payload.get("root_tests_regular_file_count").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("root_tests_readme_tracked_by_git").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    let legacy_wrapper = payload
        .get("legacy_fixture_wrapper")
        .and_then(serde_json::Value::as_object)
        .expect("legacy fixture wrapper object");
    assert_eq!(
        legacy_wrapper.get("wrapper_path").and_then(serde_json::Value::as_str),
        Some("tests/fixtures")
    );
    assert_eq!(
        legacy_wrapper.get("actual_target").and_then(serde_json::Value::as_str),
        Some("../benchmarks/tests/fixtures")
    );
    assert_eq!(legacy_wrapper.get("is_symlink").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        payload.get("roots").and_then(serde_json::Value::as_array).map(std::vec::Vec::len),
        Some(6)
    );
    assert!(payload.get("readiness_snapshots").and_then(serde_json::Value::as_array).is_some_and(
        |snapshots| snapshots.iter().any(|value| {
            value.as_str() == Some("benchmarks/readiness/all-domain-stage-tool-table.tsv")
        }) && snapshots.iter().any(|value| {
            value.as_str() == Some("benchmarks/readiness/local-ready/rendered-stage-commands.json")
        })
    ));
}
