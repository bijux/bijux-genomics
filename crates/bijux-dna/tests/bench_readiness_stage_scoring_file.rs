#![allow(clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn render_config_path(_repo_root: &Path, label: &str) -> PathBuf {
    tempfile::Builder::new()
        .prefix(label)
        .tempdir()
        .expect("temporary config directory")
        .keep()
        .join("stage-scoring.toml")
}

#[test]
fn bench_readiness_stage_scoring_writes_and_validates_governed_toml_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");
    let config_path = render_config_path(&repo_root, "stage-scoring-file-");
    let config_arg = config_path.to_string_lossy().into_owned();

    let render_output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-stage-scoring", "--output", &config_arg])
        .output()
        .expect("run render cli");
    assert!(
        render_output.status.success(),
        "render command failed: {}\nstdout:\n{}\nstderr:\n{}",
        render_output.status,
        String::from_utf8_lossy(&render_output.stdout),
        String::from_utf8_lossy(&render_output.stderr)
    );

    let printed_render_path = String::from_utf8(render_output.stdout).expect("stdout utf8");
    assert_eq!(printed_render_path.trim(), config_arg);

    let body = fs::read_to_string(&config_path).expect("read rendered TOML");
    let rendered: toml::Value = toml::from_str(&body).expect("parse rendered TOML");
    assert_eq!(
        rendered.get("schema_version").and_then(toml::Value::as_str),
        Some("bijux.bench.local_stage_scoring.v1")
    );

    let validate_output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "validate-stage-scoring", "--config", &config_arg])
        .output()
        .expect("run validate cli");
    assert!(
        validate_output.status.success(),
        "validate command failed: {}\nstdout:\n{}\nstderr:\n{}",
        validate_output.status,
        String::from_utf8_lossy(&validate_output.stdout),
        String::from_utf8_lossy(&validate_output.stderr)
    );

    let printed_validate_path = String::from_utf8(validate_output.stdout).expect("stdout utf8");
    assert_eq!(printed_validate_path.trim(), config_arg);
}

#[test]
fn bench_readiness_stage_scoring_validation_rejects_stale_toml_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");
    let config_path = render_config_path(&repo_root, "stage-scoring-stale-");
    let config_arg = config_path.to_string_lossy().into_owned();

    let render_output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-stage-scoring", "--output", &config_arg])
        .output()
        .expect("run render cli");
    assert!(
        render_output.status.success(),
        "render command failed: {}\nstdout:\n{}\nstderr:\n{}",
        render_output.status,
        String::from_utf8_lossy(&render_output.stdout),
        String::from_utf8_lossy(&render_output.stderr)
    );

    let rendered = fs::read_to_string(&config_path).expect("read rendered TOML");
    let stale = rendered.replacen("correctness = 0.35", "correctness = 0.34", 1);
    fs::write(&config_path, stale).expect("write stale TOML");

    let validate_output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "validate-stage-scoring", "--config", &config_arg])
        .output()
        .expect("run validate cli");
    assert!(
        !validate_output.status.success(),
        "validate command should reject stale TOML\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&validate_output.stdout),
        String::from_utf8_lossy(&validate_output.stderr)
    );

    let stderr = String::from_utf8_lossy(&validate_output.stderr);
    assert!(
        stderr.contains("weights summing") || stderr.contains("drifted"),
        "stale TOML failure must report the violated scoring contract, got:\n{stderr}"
    );
}
