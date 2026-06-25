#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_amplicon_micro_pipeline_writes_governed_summary_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _repo_lock =
        support::RepoProcessLock::acquire("micro-benchmark-mutators").expect("repo lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "local", "run-amplicon-micro-pipeline"])
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
        "runs/bench/micro/pipelines/amplicon/MICRO_AMPLICON_SUMMARY.json"
    );

    let summary_path = repo_root.join(rendered_path.trim());
    assert!(summary_path.is_file(), "amplicon pipeline summary must exist");

    let summary: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&summary_path).expect("read summary"))
            .expect("parse summary");
    let rows = summary.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 8);

    for row in rows {
        assert_eq!(
            row.get("status").and_then(serde_json::Value::as_str),
            Some("succeeded"),
            "amplicon micro pipeline rows must all succeed"
        );
        let evidence_path =
            row.get("evidence_path").and_then(serde_json::Value::as_str).expect("evidence path");
        assert!(
            repo_root.join(evidence_path).is_file(),
            "evidence path must exist: {evidence_path}"
        );
        let outputs =
            row.get("outputs").and_then(serde_json::Value::as_object).expect("outputs object");
        assert!(
            outputs
                .values()
                .all(|value| value.as_str().is_some_and(|path| repo_root.join(path).exists())),
            "every declared pipeline output must exist"
        );
    }
}
