#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new("cargo")
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["run", "-q", "-p", "bijux-dna", "--features", "bam_downstream", "--"])
        .args(args)
        .output()
        .expect("run cli")
}

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_validate_stage_result_accepts_generated_fake_run_manifest() {
    let fake_run_output = run_cli(&["bench", "local", "fake-run-stages", "--json"]);
    assert!(
        fake_run_output.status.success(),
        "fake-run command failed: {}\nstdout:\n{}\nstderr:\n{}",
        fake_run_output.status,
        String::from_utf8_lossy(&fake_run_output.stdout),
        String::from_utf8_lossy(&fake_run_output.stderr)
    );

    let fake_run_manifest: serde_json::Value =
        serde_json::from_slice(&fake_run_output.stdout).expect("parse fake-run manifest");
    let stage_manifest_path = fake_run_manifest
        .get("stages")
        .and_then(serde_json::Value::as_array)
        .expect("stages array")
        .iter()
        .find(|stage| {
            stage.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.report_qc")
        })
        .and_then(|stage| stage.get("stage_manifest_path").and_then(serde_json::Value::as_str))
        .expect("report_qc stage manifest path");

    let validate_output = run_cli(&[
        "bench",
        "local",
        "validate-stage-result",
        "--manifest",
        stage_manifest_path,
        "--json",
    ]);
    assert!(
        validate_output.status.success(),
        "validate-stage-result command failed: {}\nstdout:\n{}\nstderr:\n{}",
        validate_output.status,
        String::from_utf8_lossy(&validate_output.stdout),
        String::from_utf8_lossy(&validate_output.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&validate_output.stdout).expect("parse validation payload");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result_validation.v1")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some(stage_manifest_path)
    );
    assert_eq!(
        payload.get("stage_id").and_then(serde_json::Value::as_str),
        Some("fastq.report_qc")
    );
    assert_eq!(payload.get("tool_id").and_then(serde_json::Value::as_str), Some("multiqc"));
    assert_eq!(payload.get("status").and_then(serde_json::Value::as_str), Some("succeeded"));
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));
    assert!(payload
        .get("output_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|count| count >= 1));
}
