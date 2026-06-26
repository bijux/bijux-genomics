#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn render_path(repo_root: &Path, label: &str) -> (tempfile::TempDir, PathBuf) {
    let readiness_root = repo_root.join("benchmarks/readiness/hpc");
    fs::create_dir_all(&readiness_root).expect("create readiness hpc root");
    let temp_dir = tempfile::Builder::new()
        .prefix(label)
        .tempdir_in(&readiness_root)
        .expect("temporary readiness directory");
    let report_path = temp_dir.path().join("HPC_DRY_RUN_LOCAL_READY.json");
    (temp_dir, report_path)
}

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _repo_lock =
        support::RepoProcessLock::acquire("benchmark-readiness-mutators").expect("repo lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

#[test]
fn bench_local_render_hpc_dry_run_ready_proves_goals_481_to_489() {
    let repo_root = support::repo_root().expect("repo root");
    let (_temp_dir, report_path) = render_path(&repo_root, "render-hpc-dry-run-ready-");
    let report_arg = report_path.to_string_lossy().into_owned();

    let output = run_cli(&["bench", "local", "render-hpc-dry-run-ready", "--output", &report_arg]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let printed_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(
        printed_path.trim(),
        report_path
            .strip_prefix(&repo_root)
            .expect("report path relative to repo root")
            .to_string_lossy()
    );

    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&report_path).expect("read report"))
            .expect("parse report");
    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_hpc_dry_run_ready.v1")
    );
    assert_eq!(
        report.get("ready_for_first_hpc_run").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(report.get("checked_goal_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(report.get("failed_goal_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        report
            .get("behavior")
            .and_then(|value| value.get("proven"))
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        report
            .get("summary")
            .and_then(|value| value.get("candidate_job_count"))
            .and_then(serde_json::Value::as_u64),
        Some(6)
    );
    let checks = report.get("checks").and_then(serde_json::Value::as_array).expect("checks array");
    let goal_ids = checks
        .iter()
        .map(|check| check.get("goal_id").and_then(serde_json::Value::as_u64).expect("goal id"))
        .collect::<Vec<_>>();
    assert_eq!(goal_ids, vec![481, 482, 483, 484, 485, 486, 487, 488, 489]);
}
