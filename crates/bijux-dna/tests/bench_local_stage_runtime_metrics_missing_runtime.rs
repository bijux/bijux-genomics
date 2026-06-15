#![allow(clippy::expect_used, clippy::too_many_lines)]

#[cfg(feature = "bam_downstream")]
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_collect_runtime_metrics_rejects_stage_manifest_missing_runtime() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let fake_run_root = "runs/bench/local-fake-runs/stages-runtime-metrics-missing-runtime-cli";
    let fake_run_output = Command::new("cargo")
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["run", "-q", "-p", "bijux-dna", "--features", "bam_downstream", "--"])
        .args(["bench", "local", "fake-run-stages", "--output-root", fake_run_root, "--json"])
        .output()
        .expect("run fake-run cli");
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
    let absolute_stage_manifest_path = repo_root.join(stage_manifest_path);
    let mut payload: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&absolute_stage_manifest_path).expect("read stage manifest"),
    )
    .expect("parse stage manifest");
    payload.as_object_mut().expect("stage manifest object").remove("runtime");
    std::fs::write(
        &absolute_stage_manifest_path,
        serde_json::to_vec_pretty(&payload).expect("serialize broken stage manifest"),
    )
    .expect("write broken stage manifest");

    let output = Command::new("cargo")
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["run", "-q", "-p", "bijux-dna", "--features", "bam_downstream", "--"])
        .args([
            "bench",
            "local",
            "collect-runtime-metrics",
            "--fake-run-root",
            fake_run_root,
            "--output",
            "benchmarks/readiness/local-ready/runtime-metrics.missing-runtime.cli.json",
            "--json",
        ])
        .output()
        .expect("run collect-runtime-metrics cli");

    assert!(
        !output.status.success(),
        "collect-runtime-metrics should fail for missing runtime field\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("missing field `runtime`"),
        "failure should identify missing runtime field: {stderr}"
    );
}
