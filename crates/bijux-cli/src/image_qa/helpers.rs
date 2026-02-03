use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use bijux_bench::{image_qa_passed, ImageQaOutcome, ImageQaRecord};
use bijux_environment::api::{
    docker_image_exists, resolve_image, PlatformSpec, ResolvedImage, ToolImageSpec,
};
use tracing::warn;
use uuid::Uuid;

use super::QaStage;

pub(crate) fn temp_out_dir(stage: &str, tool: &str) -> Result<PathBuf> {
    let base = std::env::temp_dir().join("bijux-image-qa").join(stage);
    fs::create_dir_all(&base)?;
    let path = base.join(format!("{tool}-{}", Uuid::new_v4()));
    fs::create_dir_all(&path)?;
    Ok(path)
}

pub(crate) fn build_qa_record(
    stage: QaStage,
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    input_hash: &str,
    outcome: ImageQaOutcome,
) -> Result<ImageQaRecord> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let tool_version = spec.version.clone();
    let image_digest = spec
        .digest
        .as_ref()
        .map_or_else(|| image.full_name.clone(), ToString::to_string);
    Ok(ImageQaRecord {
        tool: tool.to_string(),
        stage: stage.stage_id().to_string(),
        tool_version,
        image_digest,
        runner: platform.runner.to_string(),
        platform: platform.name.clone(),
        input_hash: input_hash.to_string(),
        outcome,
    })
}

pub(crate) fn qa_already_passed(
    conn: &rusqlite::Connection,
    stage: QaStage,
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    input_hash: &str,
) -> Result<bool> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let image_digest = spec
        .digest
        .as_ref()
        .map_or_else(|| image.full_name.clone(), ToString::to_string);
    Ok(image_qa_passed(
        conn,
        tool,
        stage.stage_id(),
        &image_digest,
        &platform.name,
        &platform.runner.to_string(),
        input_hash,
    )?)
}

pub(crate) fn resolve_image_for_run(
    spec: &ToolImageSpec,
    platform: &PlatformSpec,
) -> Result<ResolvedImage> {
    let image = resolve_image(spec, platform)?;
    if docker_image_exists(&image) {
        return Ok(image);
    }
    if spec.digest.is_some() {
        let fallback = ResolvedImage {
            full_name: format!(
                "{}/{}:{}-{}",
                platform.image_prefix, spec.tool, spec.version, platform.arch
            ),
            arch: platform.arch.clone(),
            runner: platform.runner,
        };
        if docker_image_exists(&fallback) {
            warn!(
                "digest image missing locally; falling back to tag {}",
                fallback.full_name
            );
            return Ok(fallback);
        }
    }
    Err(anyhow!("docker image not found: {}", image.full_name))
}

pub fn ensure_image_qa_passed(
    stage: &str,
    tools: &[String],
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
) -> Result<()> {
    let cwd = std::env::current_dir().map_err(|err| anyhow!("failed to resolve cwd: {err}"))?;
    let qa_sqlite = crate::utils::image_qa_sqlite_path(&cwd, &platform.name);
    if !qa_sqlite.exists() {
        return Err(anyhow!(
            "image QA results missing; run `bijux image-qa --platform {}`",
            platform.name
        ));
    }
    let conn = bijux_bench::open_sqlite(&qa_sqlite).context("open image qa sqlite")?;
    let runner = platform.runner.to_string();
    let mut expected_inputs = bijux_bench::image_qa_inputs(&conn, stage, &platform.name, &runner)?;
    if expected_inputs.is_empty() {
        expected_inputs =
            bijux_bench::image_qa_input_hashes_from_records(&conn, stage, &platform.name, &runner)?;
    }
    if expected_inputs.is_empty() {
        return Err(anyhow!(
            "image QA inputs missing for {stage}; run `bijux image-qa --platform {}`",
            platform.name
        ));
    }
    for tool in tools {
        let spec = catalog
            .get(tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.yaml"))?;
        let image = resolve_image_for_run(spec, platform)?;
        let image_digest = spec
            .digest
            .as_ref()
            .map_or_else(|| image.full_name.clone(), ToString::to_string);
        for input_hash in &expected_inputs {
            let passed = bijux_bench::image_qa_passed(
                &conn,
                tool,
                stage,
                &image_digest,
                &platform.name,
                &runner,
                input_hash,
            )?;
            if !passed {
                return Err(anyhow!(
                    "image QA failed or missing for {tool} ({stage}); run `bijux image-qa --platform {}`",
                    platform.name
                ));
            }
        }
    }
    Ok(())
}
