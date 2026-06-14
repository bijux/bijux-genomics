#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_real_smoke_core_subset_writes_governed_summary_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "local", "run-real-smoke-core-subset"])
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
        "artifacts/benchmarks/local-real-smoke/core-subset/REAL_SMOKE_SUMMARY.json"
    );

    let summary_path = repo_root.join(rendered_path.trim());
    assert!(summary_path.is_file(), "real-smoke summary must exist");

    let summary: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&summary_path).expect("read summary"))
            .expect("parse summary");
    let rows = summary.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 4);

    for row in rows {
        let evidence_path =
            row.get("evidence_path").and_then(serde_json::Value::as_str).expect("evidence path");
        assert!(
            repo_root.join(evidence_path).is_file(),
            "evidence path must exist: {evidence_path}"
        );

        if let Some(manifest_path) =
            row.get("stage_result_manifest_path").and_then(serde_json::Value::as_str)
        {
            assert!(
                repo_root.join(manifest_path).is_file(),
                "manifest path must exist: {manifest_path}"
            );
        }
    }

    let bridge = rows
        .iter()
        .find(|row| {
            row.get("execution_id").and_then(serde_json::Value::as_str)
                == Some("bridge:bam-to-vcf.call")
        })
        .expect("bridge row");
    assert_eq!(
        bridge.get("asset_profile_id").and_then(serde_json::Value::as_str),
        Some("bam_bundle")
    );
    assert_eq!(
        bridge.get("manifest_status").and_then(serde_json::Value::as_str),
        Some("succeeded")
    );
}
