#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_fake_run_stages_writes_stage_manifests_and_declared_outputs() {
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
            "fake-run-stages",
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

    let fake_run_root = repo_root.join("target/local-fake-runs/stages");
    let root_manifest = fake_run_root.join("manifest.json");
    assert!(root_manifest.is_file(), "fake-run root manifest must exist");

    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&root_manifest).expect("read fake-run root manifest"),
    )
    .expect("parse fake-run root manifest");
    let stages =
        manifest.get("stages").and_then(serde_json::Value::as_array).expect("stages array");
    assert_eq!(stages.len(), 51);
    for stage in stages {
        let stage_manifest_path = stage
            .get("stage_manifest_path")
            .and_then(serde_json::Value::as_str)
            .expect("stage manifest path");
        assert!(repo_root.join(stage_manifest_path).is_file(), "stage manifest must exist");
    }
}
