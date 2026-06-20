#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn render_paths(repo_root: &Path, label: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let temp_dir = tempfile::Builder::new()
        .prefix(label)
        .tempdir_in(repo_root.join("runs/bench/hpc-dry-run"))
        .expect("temporary HPC dry-run directory");
    let script_path = temp_dir.path().join("stage-benchmark-array.sbatch");
    let manifest_path = temp_dir.path().join("stage-benchmark-array-manifest.json");
    (temp_dir, script_path, manifest_path)
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
fn bench_local_validate_hpc_stage_benchmark_array_prints_validated_script_path() {
    let repo_root = support::repo_root().expect("repo root");
    let (_temp_dir, script_path, _manifest_path) =
        render_paths(&repo_root, "validate-hpc-stage-benchmark-array-");
    let script_arg = script_path.to_string_lossy().into_owned();

    let render_output =
        run_cli(&["bench", "local", "render-hpc-stage-benchmark-array", "--output", &script_arg]);
    assert!(
        render_output.status.success(),
        "render command failed: {}\nstdout:\n{}\nstderr:\n{}",
        render_output.status,
        String::from_utf8_lossy(&render_output.stdout),
        String::from_utf8_lossy(&render_output.stderr)
    );

    let validate_output =
        run_cli(&["bench", "local", "validate-hpc-stage-benchmark-array", "--script", &script_arg]);
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
        script_path
            .strip_prefix(&repo_root)
            .expect("script path relative to repo root")
            .to_string_lossy()
    );
}

#[test]
fn bench_local_validate_hpc_stage_benchmark_array_rejects_stale_manifest_file() {
    let repo_root = support::repo_root().expect("repo root");
    let (_temp_dir, script_path, manifest_path) =
        render_paths(&repo_root, "validate-hpc-stage-benchmark-array-manifest-stale-");
    let script_arg = script_path.to_string_lossy().into_owned();

    let render_output =
        run_cli(&["bench", "local", "render-hpc-stage-benchmark-array", "--output", &script_arg]);
    assert!(
        render_output.status.success(),
        "render command failed: {}\nstdout:\n{}\nstderr:\n{}",
        render_output.status,
        String::from_utf8_lossy(&render_output.stdout),
        String::from_utf8_lossy(&render_output.stderr)
    );

    let stale_manifest = fs::read_to_string(&manifest_path).expect("read manifest").replacen(
        "vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools",
        "vcf:vcf_production_regression:vcf.stats:vcf_cohort:broken",
        1,
    );
    fs::write(&manifest_path, stale_manifest).expect("write stale manifest");

    let validate_output =
        run_cli(&["bench", "local", "validate-hpc-stage-benchmark-array", "--script", &script_arg]);
    assert!(
        !validate_output.status.success(),
        "validate command should reject stale manifest\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&validate_output.stdout),
        String::from_utf8_lossy(&validate_output.stderr)
    );
    let stderr = String::from_utf8_lossy(&validate_output.stderr);
    assert!(
        stderr.contains("manifest") && stderr.contains("drifted"),
        "stale manifest failure must identify manifest drift, got:\n{stderr}"
    );
}

#[test]
fn bench_local_validate_hpc_stage_benchmark_array_rejects_stale_script_file() {
    let repo_root = support::repo_root().expect("repo root");
    let (_temp_dir, script_path, _manifest_path) =
        render_paths(&repo_root, "validate-hpc-stage-benchmark-array-script-stale-");
    let script_arg = script_path.to_string_lossy().into_owned();

    let render_output =
        run_cli(&["bench", "local", "render-hpc-stage-benchmark-array", "--output", &script_arg]);
    assert!(
        render_output.status.success(),
        "render command failed: {}\nstdout:\n{}\nstderr:\n{}",
        render_output.status,
        String::from_utf8_lossy(&render_output.stdout),
        String::from_utf8_lossy(&render_output.stderr)
    );

    let stale_script = fs::read_to_string(&script_path).expect("read script").replacen(
        "#SBATCH --job-name=bijux-stage-benchmark-array",
        "#SBATCH --job-name=bijux-stage-benchmark-array-broken",
        1,
    );
    fs::write(&script_path, stale_script).expect("write stale script");

    let validate_output =
        run_cli(&["bench", "local", "validate-hpc-stage-benchmark-array", "--script", &script_arg]);
    assert!(
        !validate_output.status.success(),
        "validate command should reject stale script\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&validate_output.stdout),
        String::from_utf8_lossy(&validate_output.stderr)
    );
    let stderr = String::from_utf8_lossy(&validate_output.stderr);
    assert!(
        stderr.contains("script") && stderr.contains("drifted"),
        "stale script failure must identify script drift, got:\n{stderr}"
    );
}
