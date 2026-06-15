#![allow(clippy::expect_used, clippy::too_many_lines)]

#[cfg(feature = "bam_downstream")]
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_validate_stage_result_rejects_missing_runtime_field() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");
    let manifest_dir = tempfile::tempdir().expect("manifest tempdir");
    let manifest_path = manifest_dir.path().join("stage-result.missing-runtime.json");

    std::fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "bijux.bench.stage_result.v2",
            "stage_id": "fastq.report_qc",
            "tool": {"id": "multiqc"},
            "command": {"rendered": "echo ok"},
            "resource_metrics": {
                "source": "estimated",
                "memory_mb": 128.0,
                "cpu_threads": 1
            },
            "outputs": [{
                "artifact_id": "report_json",
                "declared_path": "declared",
                "realized_path": "realized",
                "role": "report",
                "optional": false,
                "exists": true
            }]
        }))
        .expect("serialize manifest"),
    )
    .expect("write invalid manifest");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args([
            "bench",
            "local",
            "validate-stage-result",
            "--manifest",
            manifest_path.to_str().expect("manifest path"),
            "--json",
        ])
        .output()
        .expect("run cli");

    assert!(
        !output.status.success(),
        "validate-stage-result should fail for missing runtime field\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("missing field `runtime`"),
        "failure should identify missing runtime field: {stderr}"
    );
}
