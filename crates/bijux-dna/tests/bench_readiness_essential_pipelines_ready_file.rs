#![allow(clippy::expect_used, clippy::too_many_lines)]

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_essential_pipelines_ready_writes_gate_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let sandbox =
        support::RepoSandbox::new("essential-pipelines-ready-file-").expect("repo sandbox");
    let home = tempfile::tempdir().expect("tempdir");

    let output = sandbox
        .bijux_dna_command()
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-essential-pipelines-ready"])
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
    assert_eq!(
        rendered_path.trim(),
        "artifacts/benchmark-smoke/readiness/pipelines/ESSENTIAL_PIPELINES_READY.json"
    );

    let payload: serde_json::Value = serde_json::from_slice(
        &std::fs::read(sandbox.path().join(rendered_path.trim())).expect("read readiness gate"),
    )
    .expect("parse readiness gate");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.essential_pipelines_ready.v1")
    );
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("failed_goal_count").and_then(serde_json::Value::as_u64), Some(0));
}

#[test]
fn bench_readiness_essential_pipelines_ready_persists_failed_goal_evidence() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let sandbox =
        support::RepoSandbox::new("essential-pipelines-ready-failure-").expect("repo sandbox");
    let home = tempfile::tempdir().expect("tempdir");

    std::fs::remove_file(
        sandbox.path().join("benchmarks/configs/pipelines/local/core-germline-fastq-bam-vcf.toml"),
    )
    .expect("remove one pipeline definition from sandbox");

    let output = sandbox
        .bijux_dna_command()
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-essential-pipelines-ready"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report_path = sandbox
        .path()
        .join("artifacts/benchmark-smoke/readiness/pipelines/ESSENTIAL_PIPELINES_READY.json");
    let payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(report_path).expect("read failed readiness gate"))
            .expect("parse failed readiness gate");
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(false));
    assert!(payload.get("failed_goal_count").and_then(serde_json::Value::as_u64) > Some(0));
    assert!(payload
        .get("failing_goal_ids")
        .and_then(serde_json::Value::as_array)
        .expect("failing goal ids")
        .iter()
        .any(|goal_id| goal_id.as_u64() == Some(261)));
    let failed_check = payload
        .get("checks")
        .and_then(serde_json::Value::as_array)
        .expect("checks array")
        .iter()
        .find(|check| check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(261))
        .expect("goal 261 failure check");
    assert_eq!(failed_check.get("ok").and_then(serde_json::Value::as_bool), Some(false));
    assert!(failed_check
        .get("detail")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|detail| detail.contains("No such file")));
}
