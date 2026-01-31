use std::path::Path;

use anyhow::Result;
use bijux_core::{
    ArtifactRef, CommandSpecV1, ContainerImageRefV1, StageIO, StageId, StageVersion,
    ToolConstraints, ToolId,
};
use bijux_engine::api::{execute_plan, resolve_image_for_run, StagePlanV1};
use bijux_env_runtime::api::{load_image_catalog, load_platform};
use tempfile::TempDir;

fn ensure_docker() -> bool {
    let status = std::process::Command::new("docker").arg("version").status();
    matches!(status, Ok(s) if s.success())
}

fn tempdir_in_repo() -> Result<TempDir> {
    let cwd = std::env::current_dir()?;
    let base = cwd.join("target").join("test-tmp");
    std::fs::create_dir_all(&base)?;
    Ok(TempDir::new_in(base)?)
}

#[test]
fn execute_plan_runs_trim() -> Result<()> {
    if std::env::var("BIJUX_E2E").is_err() {
        return Ok(());
    }
    if !ensure_docker() {
        eprintln!("skipping: docker not available");
        return Ok(());
    }
    let platform = load_platform(None)?;
    let catalog = load_image_catalog()?;
    let spec = catalog
        .get("fastp")
        .ok_or_else(|| anyhow::anyhow!("fastp missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, &platform)?;

    let input = Path::new("tests/data/fastq/ERR769587/ERR769587.fastq.gz").canonicalize()?;
    let out_dir = tempdir_in_repo()?;
    let output_path = out_dir.path().join("fastp.fastq.gz");

    let exec_plan = StagePlanV1 {
        stage_id: StageId("fastq.trim".to_string()),
        stage_version: StageVersion(1),
        tool_id: ToolId("fastp".to_string()),
        tool_version: spec.version.clone(),
        image: ContainerImageRefV1 {
            image: image.full_name,
            digest: spec.digest.clone(),
        },
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
                path: input,
            }],
            outputs: vec![ArtifactRef {
                name: "trimmed_reads".to_string(),
                path: output_path.clone(),
            }],
        },
        out_dir: out_dir.path().to_path_buf(),
        params: serde_json::json!({}),
        aux_images: std::collections::BTreeMap::new(),
    };
    let result = execute_plan(&exec_plan, platform.runner, None)?;
    assert_eq!(result.exit_code, 0);
    assert!(output_path.exists());
    assert!(out_dir.path().join("engine_execution.json").exists());
    Ok(())
}

#[test]
fn execute_plan_runs_validate() -> Result<()> {
    if std::env::var("BIJUX_E2E").is_err() {
        return Ok(());
    }
    if !ensure_docker() {
        eprintln!("skipping: docker not available");
        return Ok(());
    }
    let platform = load_platform(None)?;
    let catalog = load_image_catalog()?;
    let spec = catalog
        .get("fastqvalidator_official")
        .ok_or_else(|| anyhow::anyhow!("fastqvalidator_official missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, &platform)?;

    let input = Path::new("tests/data/fastq/canonical/BIJUX_SE_R1.fastq.gz").canonicalize()?;
    let out_dir = tempdir_in_repo()?;
    let exec_plan = StagePlanV1 {
        stage_id: StageId("fastq.validate_pre".to_string()),
        stage_version: StageVersion(1),
        tool_id: ToolId("fastqvalidator_official".to_string()),
        tool_version: spec.version.clone(),
        image: ContainerImageRefV1 {
            image: image.full_name,
            digest: spec.digest.clone(),
        },
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
                path: input,
            }],
            outputs: Vec::new(),
        },
        out_dir: out_dir.path().to_path_buf(),
        params: serde_json::json!({}),
        aux_images: std::collections::BTreeMap::new(),
    };
    let result = execute_plan(&exec_plan, platform.runner, None)?;
    assert_eq!(result.exit_code, 0);
    Ok(())
}

#[test]
fn execute_plan_runs_merge() -> Result<()> {
    if std::env::var("BIJUX_E2E").is_err() {
        return Ok(());
    }
    if !ensure_docker() {
        eprintln!("skipping: docker not available");
        return Ok(());
    }
    let platform = load_platform(None)?;
    let catalog = load_image_catalog()?;
    let spec = catalog
        .get("pear")
        .ok_or_else(|| anyhow::anyhow!("pear missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, &platform)?;

    let r1 = Path::new("tests/data/fastq/canonical/BIJUX_PE_R1.fastq.gz").canonicalize()?;
    let r2 = Path::new("tests/data/fastq/canonical/BIJUX_PE_R2.fastq.gz").canonicalize()?;
    let out_dir = tempdir_in_repo()?;
    let exec_plan = StagePlanV1 {
        stage_id: StageId("fastq.merge".to_string()),
        stage_version: StageVersion(1),
        tool_id: ToolId("pear".to_string()),
        tool_version: spec.version.clone(),
        image: ContainerImageRefV1 {
            image: image.full_name,
            digest: spec.digest.clone(),
        },
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
        out_dir: out_dir.path().to_path_buf(),
        params: serde_json::json!({}),
        aux_images: std::collections::BTreeMap::new(),
    };
    let result = execute_plan(&exec_plan, platform.runner, None)?;
    assert_eq!(result.exit_code, 0);
    Ok(())
}
