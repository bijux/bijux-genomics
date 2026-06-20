#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn render_manifest_path(repo_root: &Path, label: &str) -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::Builder::new()
        .prefix(label)
        .tempdir_in(repo_root.join("runs/bench/hpc-dry-run"))
        .expect("temporary HPC dry-run directory");
    let manifest_path = temp_dir.path().join("scratch-layout.json");
    (temp_dir, manifest_path)
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
fn bench_local_validate_hpc_scratch_layout_prints_validated_path() {
    let repo_root = support::repo_root().expect("repo root");
    let (_temp_dir, manifest_path) =
        render_manifest_path(&repo_root, "validate-hpc-scratch-layout-file-");
    let manifest_arg = manifest_path.to_string_lossy().into_owned();

    let render_output =
        run_cli(&["bench", "local", "render-hpc-scratch-layout", "--output", &manifest_arg]);
    assert!(
        render_output.status.success(),
        "render command failed: {}\nstdout:\n{}\nstderr:\n{}",
        render_output.status,
        String::from_utf8_lossy(&render_output.stdout),
        String::from_utf8_lossy(&render_output.stderr)
    );

    let validate_output =
        run_cli(&["bench", "local", "validate-hpc-scratch-layout", "--manifest", &manifest_arg]);
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
        manifest_path
            .strip_prefix(&repo_root)
            .expect("manifest path relative to repo root")
            .to_string_lossy()
    );
}

#[test]
fn bench_local_validate_hpc_scratch_layout_rejects_stale_manifest_file() {
    let repo_root = support::repo_root().expect("repo root");
    let (_temp_dir, manifest_path) =
        render_manifest_path(&repo_root, "validate-hpc-scratch-layout-stale-");
    let manifest_arg = manifest_path.to_string_lossy().into_owned();

    let render_output =
        run_cli(&["bench", "local", "render-hpc-scratch-layout", "--output", &manifest_arg]);
    assert!(
        render_output.status.success(),
        "render command failed: {}\nstdout:\n{}\nstderr:\n{}",
        render_output.status,
        String::from_utf8_lossy(&render_output.stdout),
        String::from_utf8_lossy(&render_output.stderr)
    );

    let rendered = serde_json::from_slice::<serde_json::Value>(
        &fs::read(&manifest_path).expect("read manifest"),
    )
    .expect("parse manifest");
    let input_link_count = rendered
        .get("input_link_count")
        .and_then(serde_json::Value::as_u64)
        .expect("input link count");
    let stale_body = fs::read_to_string(&manifest_path).expect("read manifest body").replacen(
        &format!("\"input_link_count\": {input_link_count}"),
        &format!("\"input_link_count\": {}", input_link_count.saturating_sub(1)),
        1,
    );
    fs::write(&manifest_path, stale_body).expect("write stale manifest body");

    let validate_output =
        run_cli(&["bench", "local", "validate-hpc-scratch-layout", "--manifest", &manifest_arg]);
    assert!(
        !validate_output.status.success(),
        "validate command should reject stale manifest\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&validate_output.stdout),
        String::from_utf8_lossy(&validate_output.stderr)
    );
    let stderr = String::from_utf8_lossy(&validate_output.stderr);
    assert!(
        stderr.contains("drifted"),
        "stale manifest failure must identify drift, got:\n{stderr}"
    );
}
