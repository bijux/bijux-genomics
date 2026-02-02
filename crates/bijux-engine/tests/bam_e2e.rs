use std::path::Path;

use anyhow::Result;
use bijux_core::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_engine::api::{execute_plan, resolve_image_for_run};
use bijux_env_runtime::api::{load_image_catalog, load_platform, RunnerKind};
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

fn tool_spec(tool_id: &str, runner: RunnerKind) -> Result<ToolExecutionSpecV1> {
    let catalog = load_image_catalog()?;
    let spec = catalog
        .get(tool_id)
        .ok_or_else(|| anyhow::anyhow!("{tool_id} missing from images.yaml"))?;
    let platform = load_platform(None)?;
    let image = resolve_image_for_run(spec, &platform)?;
    Ok(ToolExecutionSpecV1 {
        tool_id: ToolId(tool_id.to_string()),
        tool_version: spec.version.clone(),
        image: ContainerImageRefV1 {
            image: image.full_name,
            digest: spec.digest.clone(),
        },
        command: CommandSpecV1 {
            template: Vec::new(),
        },
        resources: ToolConstraints {
            runtime: runner.to_string(),
            mem_gb: 2,
            tmp_gb: 2,
            threads: 1,
        },
    })
}

#[test]
fn bam_mini_run_validate_qc_pre_coverage() -> Result<()> {
    if std::env::var("BIJUX_E2E").is_err() {
        return Ok(());
    }
    if !ensure_docker() {
        eprintln!("skipping: docker not available");
        return Ok(());
    }
    let input = if let Ok(path) = std::env::var("BIJUX_BAM_E2E_INPUT") {
        Path::new(&path).canonicalize()?
    } else {
        eprintln!("skipping: BIJUX_BAM_E2E_INPUT not set");
        return Ok(());
    };
    let index = std::env::var("BIJUX_BAM_E2E_BAI")
        .ok()
        .map(std::path::PathBuf::from);

    let platform = load_platform(None)?;
    let out_dir = tempdir_in_repo()?;
    let validate_tool = tool_spec("samtools", platform.runner)?;
    let qc_tool = tool_spec("samtools", platform.runner)?;
    let coverage_tool = tool_spec("mosdepth", platform.runner)?;

    let validate_plan = bijux_stages_bam::bam::validate::plan(
        &validate_tool,
        input.as_path(),
        index.as_deref(),
        None,
        out_dir.path().join("validate").as_path(),
    )?;
    let validate_result = execute_plan(&validate_plan, platform.runner, None)?;
    assert_eq!(validate_result.exit_code, 0);

    let qc_plan = bijux_stages_bam::bam::qc_pre::plan(
        &qc_tool,
        input.as_path(),
        out_dir.path().join("qc_pre").as_path(),
    )?;
    let qc_result = execute_plan(&qc_plan, platform.runner, None)?;
    assert_eq!(qc_result.exit_code, 0);

    let coverage_params = bijux_domain_bam::params::CoverageEffectiveParams {
        regions: None,
        depth_thresholds: vec![1, 3, 5],
    };
    let coverage_plan = bijux_stages_bam::bam::coverage::plan(
        &coverage_tool,
        input.as_path(),
        out_dir.path().join("coverage").as_path(),
        &coverage_params,
    )?;
    let coverage_result = execute_plan(&coverage_plan, platform.runner, None)?;
    assert_eq!(coverage_result.exit_code, 0);

    assert!(out_dir
        .path()
        .join("coverage")
        .join("coverage.mosdepth.summary.txt")
        .exists());

    Ok(())
}
