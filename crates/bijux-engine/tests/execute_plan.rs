use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::Result;
use bijux_core::{
    ArtifactRef, CommandSpecV1, ContainerImageRefV1, StageIO, StageId, StagePlanV1, StageVersion,
    ToolConstraints, ToolId,
};
use bijux_engine::api::execute_plan;
use bijux_env_runtime::api::RunnerKind;
use flate2::write::GzEncoder;
use flate2::Compression;
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

fn write_gzip(path: &Path, contents: &str) -> Result<()> {
    let file = fs::File::create(path)?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(contents.as_bytes())?;
    encoder.finish()?;
    Ok(())
}

fn test_image() -> ContainerImageRefV1 {
    ContainerImageRefV1 {
        image: "bijux/test:latest".to_string(),
        digest: None,
    }
}

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn execute_plan_success_path_uses_public_api() -> Result<()> {
    let _guard = ENV_LOCK
        .lock()
        .map_err(|_| anyhow::anyhow!("env lock poisoned"))?;
    let (dir, r1, _r2) = temp_inputs()?;
    let bin_dir = write_fake_docker(dir.path())?;
    let log_path = dir.path().join("docker.log");
    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("BIJUX_TEST_DOCKER_LOG", &log_path);
    std::env::set_var("PATH", format!("{}:{}", bin_dir.display(), original_path));

    let out_dir = dir.path().join("out");
    fs::create_dir_all(&out_dir)?;
    let output_path = out_dir.join("fastp.fastq.gz");
    write_gzip(&output_path, "@r1\nACGT\n+\n!!!!\n")?;
    let exec_plan = StagePlanV1 {
        stage_id: StageId("fastq.trim".to_string()),
        stage_version: StageVersion(1),
        tool_id: ToolId("fastp".to_string()),
        tool_version: "1.0.0".to_string(),
        image: test_image(),
        command: CommandSpecV1 {
            template: Vec::new(),
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
        io: StageIO {
            inputs: vec![ArtifactRef {
                name: "reads_r1".to_string(),
                path: r1,
            }],
            outputs: vec![ArtifactRef {
                name: "trimmed_reads".to_string(),
                path: output_path.clone(),
            }],
        },
        out_dir: out_dir.clone(),
        params: serde_json::json!({}),
        aux_images: std::collections::BTreeMap::new(),
    };
    let result = execute_plan(&exec_plan, RunnerKind::Docker, None)?;
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.outputs, vec![output_path]);
    assert!(out_dir.join("engine_execution.json").exists());

    std::env::remove_var("BIJUX_TEST_DOCKER_LOG");
    std::env::set_var("PATH", original_path);
    Ok(())
}

#[test]
fn execute_plan_propagates_tool_failure() -> Result<()> {
    let _guard = ENV_LOCK
        .lock()
        .map_err(|_| anyhow::anyhow!("env lock poisoned"))?;
    let (dir, r1, _r2) = temp_inputs()?;
    let bin_dir = write_fake_docker(dir.path())?;
    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("BIJUX_TEST_DOCKER_EXIT_CODE", "7");
    std::env::set_var("PATH", format!("{}:{}", bin_dir.display(), original_path));

    let out_dir = dir.path().join("out_fail");
    fs::create_dir_all(&out_dir)?;
    let output_path = out_dir.join("fastp.fastq.gz");
    let exec_plan = StagePlanV1 {
        stage_id: StageId("fastq.trim".to_string()),
        stage_version: StageVersion(1),
        tool_id: ToolId("fastp".to_string()),
        tool_version: "1.0.0".to_string(),
        image: test_image(),
        command: CommandSpecV1 {
            template: Vec::new(),
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
        io: StageIO {
            inputs: vec![ArtifactRef {
                name: "reads_r1".to_string(),
                path: r1,
            }],
            outputs: vec![ArtifactRef {
                name: "trimmed_reads".to_string(),
                path: output_path,
            }],
        },
        out_dir: out_dir.clone(),
        params: serde_json::json!({}),
        aux_images: std::collections::BTreeMap::new(),
    };
    let result = execute_plan(&exec_plan, RunnerKind::Docker, None)?;
    assert_eq!(result.exit_code, 1);

    std::env::remove_var("BIJUX_TEST_DOCKER_EXIT_CODE");
    std::env::set_var("PATH", original_path);
    Ok(())
}

#[test]
fn execute_plan_hits_validate_path() -> Result<()> {
    let _guard = ENV_LOCK
        .lock()
        .map_err(|_| anyhow::anyhow!("env lock poisoned"))?;
    let (dir, r1, _r2) = temp_inputs()?;
    let bin_dir = write_fake_docker(dir.path())?;
    let log_path = dir.path().join("docker.log");
    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("BIJUX_TEST_DOCKER_LOG", &log_path);
    std::env::set_var("PATH", format!("{}:{}", bin_dir.display(), original_path));

    let out_dir = dir.path().join("validate");
    fs::create_dir_all(&out_dir)?;
    let exec_plan = StagePlanV1 {
        stage_id: StageId("fastq.validate_pre".to_string()),
        stage_version: StageVersion(1),
        tool_id: ToolId("fastqvalidator_official".to_string()),
        tool_version: "1.0.0".to_string(),
        image: test_image(),
        command: CommandSpecV1 {
            template: Vec::new(),
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
        io: StageIO {
            inputs: vec![ArtifactRef {
                name: "reads_r1".to_string(),
                path: r1,
            }],
            outputs: Vec::new(),
        },
        out_dir: out_dir.clone(),
        params: serde_json::json!({}),
        aux_images: std::collections::BTreeMap::new(),
    };
    let result = execute_plan(&exec_plan, RunnerKind::Docker, None)?;
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
    let _guard = ENV_LOCK
        .lock()
        .map_err(|_| anyhow::anyhow!("env lock poisoned"))?;
    let (dir, r1, r2) = temp_inputs()?;
    let bin_dir = write_fake_docker(dir.path())?;
    let log_path = dir.path().join("docker.log");
    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("BIJUX_TEST_DOCKER_LOG", &log_path);
    std::env::set_var("PATH", format!("{}:{}", bin_dir.display(), original_path));

    let out_dir = dir.path().join("merge");
    fs::create_dir_all(&out_dir)?;
    fs::write(out_dir.join("pear.assembled.fastq"), "@r1\nACGT\n+\n!!!!\n")?;
    fs::write(
        out_dir.join("pear.unassembled.forward.fastq"),
        "@r1\nACGT\n+\n!!!!\n",
    )?;
    fs::write(
        out_dir.join("pear.unassembled.reverse.fastq"),
        "@r2\nTGCA\n+\n!!!!\n",
    )?;
    let exec_plan = StagePlanV1 {
        stage_id: StageId("fastq.merge".to_string()),
        stage_version: StageVersion(1),
        tool_id: ToolId("pear".to_string()),
        tool_version: "1.0.0".to_string(),
        image: test_image(),
        command: CommandSpecV1 {
            template: Vec::new(),
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
        io: StageIO {
            inputs: vec![
                ArtifactRef {
                    name: "reads_r1".to_string(),
                    path: r1,
                },
                ArtifactRef {
                    name: "reads_r2".to_string(),
                    path: r2,
                },
            ],
            outputs: Vec::new(),
        },
        out_dir: out_dir.clone(),
        params: serde_json::json!({}),
        aux_images: std::collections::BTreeMap::new(),
    };
    let result = execute_plan(&exec_plan, RunnerKind::Docker, None)?;
    assert_eq!(result.exit_code, 0);
    assert!(result.command.contains("pear"));
    let log = fs::read_to_string(&log_path)?;
    assert!(log.contains("run"));

    std::env::remove_var("BIJUX_TEST_DOCKER_LOG");
    std::env::set_var("PATH", original_path);
    Ok(())
}
