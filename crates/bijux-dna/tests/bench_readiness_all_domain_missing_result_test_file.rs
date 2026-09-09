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
fn bench_readiness_all_domain_missing_result_audit_writes_governed_report_and_fixture_tree() {
    let (sandbox, payload) = run_cli_json_with_sandbox(&[
        "bench",
        "readiness",
        "render-all-domain-missing-result-test",
        "--json",
    ]);
    let repo_root = sandbox.path();

    let report_path =
        repo_root.join("artifacts/benchmark-smoke/readiness/missing-result-test-all-domains.json");
    assert!(report_path.is_file(), "all-domain missing-result report must exist");

    let persisted: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).expect("read missing-result report"))
            .expect("parse missing-result report");
    assert_eq!(
        persisted.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/missing-result-test-all-domains.json")
    );
    assert_eq!(
        persisted.get("missing_result_row_count").and_then(serde_json::Value::as_u64),
        Some(3)
    );
    let expected_row_count = persisted
        .get("expected_row_count")
        .and_then(serde_json::Value::as_u64)
        .expect("expected_row_count");
    let present_result_row_count = persisted
        .get("present_result_row_count")
        .and_then(serde_json::Value::as_u64)
        .expect("present_result_row_count");
    assert_eq!(present_result_row_count + 3, expected_row_count);

    let fixture_root = repo_root.join(
        "artifacts/benchmark-smoke/simulated-runs/readiness-probes/all-domains/missing-result-test",
    );
    assert!(fixture_root.is_dir(), "all-domain missing-result fixture root must exist");
    assert!(
        fixture_root.join("manifest.json").is_file(),
        "all-domain fake-run manifest must exist"
    );

    let removed_manifest_paths = payload
        .get("removed_manifest_paths")
        .and_then(serde_json::Value::as_array)
        .expect("removed manifest paths");
    assert_eq!(removed_manifest_paths.len(), 3);
    for path in removed_manifest_paths {
        let absolute = repo_root.join(path.as_str().expect("manifest path"));
        assert!(!absolute.exists(), "removed manifest `{}` must stay absent", absolute.display());
    }

    let rows = persisted.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    let present_manifest = rows
        .iter()
        .find(|row| row.get("result_status").and_then(serde_json::Value::as_str) == Some("present"))
        .and_then(|row| row.get("audit_manifest_path").and_then(serde_json::Value::as_str))
        .expect("present manifest path");
    assert!(
        repo_root.join(present_manifest).is_file(),
        "present rows must keep a realized manifest path"
    );
}
