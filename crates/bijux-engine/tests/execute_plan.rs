use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::Result;
use bijux_engine::api::{execute_plan, StagePlan};
use bijux_environment::api::{ResolvedImage, RunnerKind};
use tempfile::TempDir;

fn write_fake_docker(dir: &Path) -> Result<PathBuf> {
    let bin_dir = dir.join("bin");
    fs::create_dir_all(&bin_dir)?;
    let docker_path = bin_dir.join("docker");
    let script = r#"#!/bin/sh
set -e
if [ -n "$BIJUX_TEST_DOCKER_LOG" ]; then
  echo "$@" >> "$BIJUX_TEST_DOCKER_LOG"
fi
cmd="$1"
shift || true
case "$cmd" in
  run)
    echo "fake-container-id"
    exit 0
    ;;
  wait)
    echo "${BIJUX_TEST_DOCKER_EXIT_CODE:-0}"
    exit 0
    ;;
  logs)
    echo "fake logs"
    exit 0
    ;;
  stats)
    echo "10MiB / 100MiB"
    exit 0
    ;;
  rm)
    exit 0
    ;;
  inspect)
    echo "exited"
    exit 0
    ;;
  *)
    exit 0
    ;;
 esac
"#;
    fs::write(&docker_path, script)?;
    let mut perms = fs::metadata(&docker_path)?.permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
        fs::set_permissions(&docker_path, perms)?;
    }
    Ok(bin_dir)
}

fn temp_inputs() -> Result<(TempDir, PathBuf, PathBuf)> {
    let dir = TempDir::new()?;
    let r1 = dir.path().join("input_r1.fastq");
    let r2 = dir.path().join("input_r2.fastq");
    fs::write(&r1, "@r1\nACGT\n+\n!!!!\n")?;
    fs::write(&r2, "@r2\nTGCA\n+\n!!!!\n")?;
    Ok((dir, r1, r2))
}

fn test_image() -> ResolvedImage {
    ResolvedImage {
        full_name: "bijux/test:latest".to_string(),
        arch: "amd64".to_string(),
        runner: RunnerKind::Docker,
    }
}

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn execute_plan_success_path_uses_public_api() -> Result<()> {
    let _guard = ENV_LOCK.lock().unwrap();
    let (dir, r1, _r2) = temp_inputs()?;
    let bin_dir = write_fake_docker(dir.path())?;
    let log_path = dir.path().join("docker.log");
    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("BIJUX_TEST_DOCKER_LOG", &log_path);
    std::env::set_var("PATH", format!("{}:{}", bin_dir.display(), original_path));

    let out_dir = dir.path().join("out");
    fs::create_dir_all(&out_dir)?;
    let output_path = out_dir.join("fastp.fastq.gz");
    let exec_plan = StagePlan {
        stage_id: "fastq.trim".to_string(),
        tool: "fastp".to_string(),
        image: test_image(),
        runner: RunnerKind::Docker,
        inputs: vec![r1],
        out_dir: out_dir.clone(),
        outputs: vec![output_path.clone()],
        params: serde_json::json!({}),
        aux_images: HashMap::new(),
    };
    let result = execute_plan(&exec_plan)?;
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.outputs, vec![output_path]);
    assert!(out_dir.join("engine_execution.json").exists());

    std::env::remove_var("BIJUX_TEST_DOCKER_LOG");
    std::env::set_var("PATH", original_path);
    Ok(())
}

#[test]
fn execute_plan_propagates_tool_failure() -> Result<()> {
    let _guard = ENV_LOCK.lock().unwrap();
    let (dir, r1, _r2) = temp_inputs()?;
    let bin_dir = write_fake_docker(dir.path())?;
    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("BIJUX_TEST_DOCKER_EXIT_CODE", "7");
    std::env::set_var("PATH", format!("{}:{}", bin_dir.display(), original_path));

    let out_dir = dir.path().join("out_fail");
    fs::create_dir_all(&out_dir)?;
    let exec_plan = StagePlan {
        stage_id: "fastq.trim".to_string(),
        tool: "fastp".to_string(),
        image: test_image(),
        runner: RunnerKind::Docker,
        inputs: vec![r1],
        out_dir: out_dir.clone(),
        outputs: vec![out_dir.join("fastp.fastq.gz")],
        params: serde_json::json!({}),
        aux_images: HashMap::new(),
    };
    let result = execute_plan(&exec_plan)?;
    assert_eq!(result.exit_code, 7);

    std::env::remove_var("BIJUX_TEST_DOCKER_EXIT_CODE");
    std::env::set_var("PATH", original_path);
    Ok(())
}

#[test]
fn execute_plan_hits_validate_path() -> Result<()> {
    let _guard = ENV_LOCK.lock().unwrap();
    let (dir, r1, _r2) = temp_inputs()?;
    let bin_dir = write_fake_docker(dir.path())?;
    let log_path = dir.path().join("docker.log");
    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("BIJUX_TEST_DOCKER_LOG", &log_path);
    std::env::set_var("PATH", format!("{}:{}", bin_dir.display(), original_path));

    let out_dir = dir.path().join("validate");
    fs::create_dir_all(&out_dir)?;
    let exec_plan = StagePlan {
        stage_id: "fastq.validate_pre".to_string(),
        tool: "fastqvalidator_official".to_string(),
        image: test_image(),
        runner: RunnerKind::Docker,
        inputs: vec![r1],
        out_dir: out_dir.clone(),
        outputs: Vec::new(),
        params: serde_json::json!({}),
        aux_images: HashMap::new(),
    };
    let result = execute_plan(&exec_plan)?;
    assert_eq!(result.exit_code, 0);
    assert!(result.command.contains("fastq-validator"));
    let log = fs::read_to_string(&log_path)?;
    assert!(log.contains("run"));

    std::env::remove_var("BIJUX_TEST_DOCKER_LOG");
    std::env::set_var("PATH", original_path);
    Ok(())
}

#[test]
fn execute_plan_hits_merge_path() -> Result<()> {
    let _guard = ENV_LOCK.lock().unwrap();
    let (dir, r1, r2) = temp_inputs()?;
    let bin_dir = write_fake_docker(dir.path())?;
    let log_path = dir.path().join("docker.log");
    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("BIJUX_TEST_DOCKER_LOG", &log_path);
    std::env::set_var("PATH", format!("{}:{}", bin_dir.display(), original_path));

    let out_dir = dir.path().join("merge");
    fs::create_dir_all(&out_dir)?;
    let exec_plan = StagePlan {
        stage_id: "fastq.merge".to_string(),
        tool: "pear".to_string(),
        image: test_image(),
        runner: RunnerKind::Docker,
        inputs: vec![r1, r2],
        out_dir: out_dir.clone(),
        outputs: Vec::new(),
        params: serde_json::json!({}),
        aux_images: HashMap::new(),
    };
    let result = execute_plan(&exec_plan)?;
    assert_eq!(result.exit_code, 0);
    assert!(result.command.contains("pear"));
    let log = fs::read_to_string(&log_path)?;
    assert!(log.contains("run"));

    std::env::remove_var("BIJUX_TEST_DOCKER_LOG");
    std::env::set_var("PATH", original_path);
    Ok(())
}
