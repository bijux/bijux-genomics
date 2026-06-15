#![allow(clippy::expect_used)]

#[cfg(feature = "bam_downstream")]
#[cfg(feature = "bam_downstream")]
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_fake_run_failures_writes_failure_records_and_stderr_files() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new("cargo")
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args([
            "run",
            "-q",
            "-p",
            "bijux-dna",
            "--features",
            "bam_downstream",
            "--",
            "bench",
            "local",
            "fake-run-failures",
            "--stage-id",
            "fastq.report_qc",
            "--stage-id",
            "bam.validate",
            "--exit-code",
            "9",
        ])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let failure_root = repo_root.join("runs/bench/local-fake-runs/failures");
    let root_manifest = failure_root.join("manifest.json");
    assert!(root_manifest.is_file(), "fake failure root manifest must exist");

    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&root_manifest).expect("read fake failure root manifest"),
    )
    .expect("parse fake failure root manifest");
    let failures =
        manifest.get("failures").and_then(serde_json::Value::as_array).expect("failures array");
    assert_eq!(failures.len(), 2);
    for failure in failures {
        let stderr_path =
            failure.get("stderr_path").and_then(serde_json::Value::as_str).expect("stderr path");
        let failure_record_path = failure
            .get("failure_record_path")
            .and_then(serde_json::Value::as_str)
            .expect("failure record path");
        let failed_output_count = failure
            .get("failed_output_count")
            .and_then(serde_json::Value::as_u64)
            .expect("failed output count");
        assert!(repo_root.join(stderr_path).is_file(), "stderr file must exist");
        assert!(repo_root.join(failure_record_path).is_file(), "failure record file must exist");
        assert!(failed_output_count >= 1, "failure record must list missing outputs");
    }
}
