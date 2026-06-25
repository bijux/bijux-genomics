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
    let report_path = temp_dir.path().join("FIRST_HPC_CANDIDATE_RUN.json");
    (temp_dir, report_path)
}

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
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
fn bench_local_validate_hpc_candidate_run_manifest_prints_validated_report_path() {
    let repo_root = support::repo_root().expect("repo root");
    let (_temp_dir, report_path) = render_path(&repo_root, "validate-hpc-candidate-run-manifest-");
    let report_arg = report_path.to_string_lossy().into_owned();

    let render_output =
        run_cli(&["bench", "local", "render-hpc-candidate-run-manifest", "--output", &report_arg]);
    assert!(
        render_output.status.success(),
        "render command failed: {}\nstdout:\n{}\nstderr:\n{}",
        render_output.status,
        String::from_utf8_lossy(&render_output.stdout),
        String::from_utf8_lossy(&render_output.stderr)
    );

    let validate_output = run_cli(&[
        "bench",
        "local",
        "validate-hpc-candidate-run-manifest",
        "--manifest",
        &report_arg,
    ]);
    assert!(
        validate_output.status.success(),
        "validate command failed: {}\nstdout:\n{}\nstderr:\n{}",
        validate_output.status,
        String::from_utf8_lossy(&validate_output.stdout),
        String::from_utf8_lossy(&validate_output.stderr)
    );

    let printed_path = String::from_utf8(validate_output.stdout).expect("stdout utf8");
    assert_eq!(
        printed_path.trim(),
        report_path
            .strip_prefix(&repo_root)
            .expect("report path relative to repo root")
            .to_string_lossy()
    );
}

#[test]
fn bench_local_validate_hpc_candidate_run_manifest_rejects_stale_report_file() {
    let repo_root = support::repo_root().expect("repo root");
    let (_temp_dir, report_path) =
        render_path(&repo_root, "validate-hpc-candidate-run-manifest-stale-");
    let report_arg = report_path.to_string_lossy().into_owned();

    let render_output =
        run_cli(&["bench", "local", "render-hpc-candidate-run-manifest", "--output", &report_arg]);
    assert!(
        render_output.status.success(),
        "render command failed: {}\nstdout:\n{}\nstderr:\n{}",
        render_output.status,
        String::from_utf8_lossy(&render_output.stdout),
        String::from_utf8_lossy(&render_output.stderr)
    );

    let stale_report = fs::read_to_string(&report_path).expect("read report").replacen(
        "\"selected_job_count\": 6",
        "\"selected_job_count\": 7",
        1,
    );
    fs::write(&report_path, stale_report).expect("write stale report");

    let validate_output = run_cli(&[
        "bench",
        "local",
        "validate-hpc-candidate-run-manifest",
        "--manifest",
        &report_arg,
    ]);
    assert!(
        !validate_output.status.success(),
        "validate command should reject stale report\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&validate_output.stdout),
        String::from_utf8_lossy(&validate_output.stderr)
    );
    let stderr = String::from_utf8_lossy(&validate_output.stderr);
    assert!(
        stderr.contains("candidate run manifest") && stderr.contains("drifted"),
        "stale report failure must identify candidate-manifest drift, got:\n{stderr}"
    );
}
