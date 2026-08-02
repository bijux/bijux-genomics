#![allow(clippy::expect_used, clippy::too_many_lines)]

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_paths_validate_writes_governed_validation_report() {
    let sandbox =
        support::RepoSandbox::new("benchmark-path-validation-file-").expect("repo sandbox");
    let repo_root = sandbox.path();
    let home = tempfile::tempdir().expect("tempdir");

    let schema_output = sandbox
        .bijux_dna_command()
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args([
            "bench",
            "validate-schemas",
            "--schema-root",
            "benchmarks/schemas",
            "--domain",
            "fastq,bam,vcf",
            "--json",
        ])
        .output()
        .expect("run schema validation cli");

    assert!(
        schema_output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        schema_output.status,
        String::from_utf8_lossy(&schema_output.stdout),
        String::from_utf8_lossy(&schema_output.stderr)
    );

    let stage_tool_output = sandbox
        .bijux_dna_command()
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-all-domain-stage-tool-table", "--json"])
        .output()
        .expect("run stage tool table cli");

    assert!(
        stage_tool_output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        stage_tool_output.status,
        String::from_utf8_lossy(&stage_tool_output.stdout),
        String::from_utf8_lossy(&stage_tool_output.stderr)
    );

    let output = sandbox
        .bijux_dna_command()
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "paths", "validate", "--strict"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/benchmark-paths-validation.json");

    let payload: serde_json::Value = serde_json::from_slice(
        &std::fs::read(repo_root.join(rendered_path.trim()))
            .expect("read benchmark paths validation report"),
    )
    .expect("parse benchmark paths validation report");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.paths_validate.v1")
    );
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("violation_count").and_then(serde_json::Value::as_u64), Some(0));
    assert!(payload
        .get("readiness_tsv_snapshot_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count >= 1));
    assert!(payload
        .get("readiness_json_snapshot_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count >= 1));
    let readiness_snapshots = payload
        .get("readiness_snapshots")
        .and_then(serde_json::Value::as_array)
        .expect("readiness snapshots array");
    assert!(readiness_snapshots.iter().any(|value| {
        value.as_str() == Some("benchmarks/readiness/all-domain-stage-tool-table.tsv")
    }));

    let target_root = repo_root.join("target");
    if target_root.exists() {
        std::fs::remove_dir_all(&target_root).expect("remove target");
    }

    assert!(repo_root.join("benchmarks/readiness/benchmark-paths-validation.json").is_file());
    assert!(repo_root.join("benchmarks/readiness/all-domain-stage-tool-table.tsv").is_file());

    let rerun_output = sandbox
        .bijux_dna_command()
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "paths", "validate", "--strict", "--json"])
        .output()
        .expect("rerun cli");

    assert!(
        rerun_output.status.success(),
        "command failed after removing target: {}\nstdout:\n{}\nstderr:\n{}",
        rerun_output.status,
        String::from_utf8_lossy(&rerun_output.stdout),
        String::from_utf8_lossy(&rerun_output.stderr)
    );

    let rerun_payload: serde_json::Value =
        serde_json::from_slice(&rerun_output.stdout).expect("parse rerun stdout as json");
    assert_eq!(
        rerun_payload
            .get("legacy_fixture_wrapper")
            .and_then(|value| value.get("wrapper_path"))
            .and_then(serde_json::Value::as_str),
        Some("tests/fixtures")
    );
    assert_eq!(
        rerun_payload.get("readiness_snapshot_count").and_then(serde_json::Value::as_u64),
        payload.get("readiness_snapshot_count").and_then(serde_json::Value::as_u64)
    );
    assert_eq!(
        rerun_payload
            .get("legacy_fixture_wrapper")
            .and_then(|value| value.get("actual_target"))
            .and_then(serde_json::Value::as_str),
        Some("../benchmarks/tests/fixtures")
    );
}
