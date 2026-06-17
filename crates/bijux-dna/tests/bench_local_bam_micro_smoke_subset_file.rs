#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_bam_micro_smoke_subset_writes_governed_summary_file() {
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
        .args(["bench", "local", "run-bam-micro-smoke-subset"])
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
    assert_eq!(rendered_path.trim(), "runs/bench/micro/bam/BAM_MICRO_SMOKE_SUMMARY.json");

    let summary_path = repo_root.join(rendered_path.trim());
    assert!(summary_path.is_file(), "micro-smoke summary must exist");

    let summary: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&summary_path).expect("read summary"))
            .expect("parse summary");
    let rows = summary.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 12);

    for row in rows {
        let execution_status = row
            .get("execution_status")
            .and_then(serde_json::Value::as_str)
            .expect("execution status");
        let support_path = row
            .get("smoke_support_path")
            .and_then(serde_json::Value::as_str)
            .expect("support path");
        assert!(
            repo_root.join(support_path).exists(),
            "smoke support path must exist: {support_path}"
        );

        match execution_status {
            "local_smoke" => {
                let evidence_path = row
                    .get("evidence_path")
                    .and_then(serde_json::Value::as_str)
                    .expect("evidence path");
                assert!(
                    repo_root.join(evidence_path).is_file(),
                    "local-smoke evidence path must exist: {evidence_path}"
                );
            }
            "container_needed" => {
                assert!(
                    row.get("evidence_path").is_some_and(serde_json::Value::is_null),
                    "container-needed rows must not claim local evidence"
                );
            }
            other => panic!("unexpected execution_status `{other}`"),
        }
    }
}
