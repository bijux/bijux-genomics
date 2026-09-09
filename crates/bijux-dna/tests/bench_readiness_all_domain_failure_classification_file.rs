#![allow(clippy::expect_used, clippy::too_many_lines)]

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json_with_sandbox(args: &[&str]) -> (support::RepoSandbox, serde_json::Value) {
    let sandbox = support::RepoSandbox::new("readiness-report-files-").expect("repository sandbox");
    let home = tempfile::tempdir().expect("home directory");
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
    (sandbox, serde_json::from_slice(&output.stdout).expect("parse stdout as json"))
}

#[test]
fn bench_readiness_all_domain_failure_classification_writes_governed_report_and_probe_fixture() {
    let (sandbox, payload) = run_cli_json_with_sandbox(&[
        "bench",
        "readiness",
        "render-all-domain-failure-classification",
        "--json",
    ]);
    let repo_root = sandbox.path();

    let report_path = repo_root
        .join("artifacts/benchmark-smoke/readiness/failure-classification-all-domains.json");
    assert!(report_path.is_file(), "all-domain failure-classification report must exist");

    let persisted: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&report_path).expect("read all-domain failure-classification report"),
    )
    .expect("parse all-domain failure-classification report");
    assert_eq!(
        persisted.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.all_domain_failure_classification.v1")
    );
    assert_eq!(persisted.get("passes_behavior_test"), Some(&serde_json::Value::Bool(true)));

    let fixture_root =
        repo_root.join("artifacts/benchmark-smoke/simulated-runs/readiness-probes/all-domains/failure-classification");
    assert!(fixture_root.is_dir(), "failure-classification fixture root must exist");

    let rows = persisted.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    let tool_not_found = rows
        .iter()
        .find(|row| {
            row.get("class_id").and_then(serde_json::Value::as_str) == Some("tool_not_found")
        })
        .expect("tool not found row");
    let command_script_path = repo_root.join(
        tool_not_found
            .get("evidence_path")
            .and_then(serde_json::Value::as_str)
            .expect("tool-not-found evidence path"),
    );
    assert!(command_script_path.is_file(), "tool-not-found probe command script must exist");

    let unsupported_pair = rows
        .iter()
        .find(|row| {
            row.get("class_id").and_then(serde_json::Value::as_str) == Some("unsupported_pair")
        })
        .expect("unsupported pair row");
    let unsupported_pair_evidence = repo_root.join(
        unsupported_pair
            .get("evidence_path")
            .and_then(serde_json::Value::as_str)
            .expect("unsupported-pair evidence path"),
    );
    assert!(
        unsupported_pair_evidence.is_file(),
        "unsupported-pair evidence must point to the governed all-domain stage-tool table"
    );

    let missing_output = payload
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .expect("payload rows")
        .iter()
        .find(|row| {
            row.get("class_id").and_then(serde_json::Value::as_str) == Some("missing_output")
        })
        .expect("missing output row");
    let missing_output_artifact = repo_root.join(
        missing_output
            .get("evidence_path")
            .and_then(serde_json::Value::as_str)
            .expect("missing-output evidence path"),
    );
    assert!(
        !missing_output_artifact.exists(),
        "missing-output evidence path must stay absent because the governed probe never creates the declared output"
    );
}
