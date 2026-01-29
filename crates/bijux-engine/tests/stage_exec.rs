use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use bijux_engine::api::{execute_plan, resolve_image_for_run, StagePlan};
use bijux_environment::api::{load_image_catalog, load_platform};
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

    let exec_plan = StagePlan {
        stage_id: "fastq.trim".to_string(),
        tool: "fastp".to_string(),
        image,
        runner: platform.runner,
        inputs: vec![input],
        out_dir: out_dir.path().to_path_buf(),
        outputs: vec![output_path.clone()],
        params: serde_json::json!({}),
        aux_images: HashMap::new(),
    };
    let result = execute_plan(&exec_plan)?;
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
    let exec_plan = StagePlan {
        stage_id: "fastq.validate_pre".to_string(),
        tool: "fastqvalidator_official".to_string(),
        image,
        runner: platform.runner,
        inputs: vec![input],
        out_dir: out_dir.path().to_path_buf(),
        outputs: Vec::new(),
        params: serde_json::json!({}),
        aux_images: HashMap::new(),
    };
    let result = execute_plan(&exec_plan)?;
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
    let exec_plan = StagePlan {
        stage_id: "fastq.merge".to_string(),
        tool: "pear".to_string(),
        image,
        runner: platform.runner,
        inputs: vec![r1, r2],
        out_dir: out_dir.path().to_path_buf(),
        outputs: Vec::new(),
        params: serde_json::json!({}),
        aux_images: HashMap::new(),
    };
    let result = execute_plan(&exec_plan)?;
    assert_eq!(result.exit_code, 0);
    Ok(())
}
