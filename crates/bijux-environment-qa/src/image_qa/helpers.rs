use std::collections::HashMap;
use std::path::PathBuf;

use crate::api::{PlatformSpec, ToolImageSpec};
use anyhow::{anyhow, Context, Result};
use bijux_analyze::{image_qa_passed, ImageQaOutcome, ImageQaRecord};
use uuid::Uuid;

use super::support::resolve_image_for_run;
use super::QaStage;

pub(crate) fn temp_out_dir(stage: &str, tool: &str) -> Result<PathBuf> {
    let base = std::env::temp_dir().join("bijux-image-qa").join(stage);
    bijux_infra::ensure_dir(&base)?;
    let path = base.join(format!("{tool}-{}", Uuid::new_v4()));
    bijux_infra::ensure_dir(&path)?;
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
    let stage_id = stage.stage_id();
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.toml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let tool_version = spec.version.clone();
    let image_digest = spec
        .digest
        .as_ref()
        .map_or_else(|| image.full_name.clone(), ToString::to_string);
    Ok(ImageQaRecord {
        tool: tool.to_string(),
        stage: stage_id.as_str().to_string(),
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
    let stage_id = stage.stage_id();
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.toml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let image_digest = spec
        .digest
        .as_ref()
        .map_or_else(|| image.full_name.clone(), ToString::to_string);
    image_qa_passed(
        conn,
        tool,
        stage_id.as_str(),
        &image_digest,
        &platform.name,
        &platform.runner.to_string(),
        input_hash,
    )
}

/// Ensure required image QA results exist for a stage/tool set.
///
/// # Errors
/// Returns an error if QA results are missing or cannot be read.
pub fn ensure_image_qa_passed<S: ::std::hash::BuildHasher>(
    stage: &str,
    tools: &[String],
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec, S>,
) -> Result<()> {
    let cwd = std::env::current_dir().map_err(|err| anyhow!("failed to resolve cwd: {err}"))?;
    let qa_sqlite = super::support::image_qa_sqlite_path(&cwd, &platform.name);
    if !qa_sqlite.exists() {
        return Err(anyhow!(
            "image QA results missing; run `bijux image-qa --platform {}`",
            platform.name
        ));
    }
    let conn = bijux_analyze::open_sqlite(&qa_sqlite).context("open image qa sqlite")?;
    let runner = platform.runner.to_string();
    let mut expected_inputs =
        bijux_analyze::image_qa_inputs(&conn, stage, &platform.name, &runner)?;
    if expected_inputs.is_empty() {
        expected_inputs = bijux_analyze::image_qa_input_hashes_from_records(
            &conn,
            stage,
            &platform.name,
            &runner,
        )?;
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
            .ok_or_else(|| anyhow!("tool {tool} missing from images.toml"))?;
        let image = resolve_image_for_run(spec, platform)?;
        let image_digest = spec
            .digest
            .as_ref()
            .map_or_else(|| image.full_name.clone(), ToString::to_string);
        for input_hash in &expected_inputs {
            let passed = bijux_analyze::image_qa_passed(
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

/// Ensure tool-level QA results exist for a stage/tool set.
///
/// # Errors
/// Returns an error if QA results are missing or cannot be read.
pub fn ensure_tool_qa_passed<S: ::std::hash::BuildHasher>(
    stage: &str,
    tools: &[String],
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec, S>,
) -> Result<()> {
    ensure_image_qa_passed(stage, tools, platform, catalog).map_err(|err| {
        anyhow!(
            "tool QA failed or missing for {stage}; run `bijux image-qa --platform {}`: {err}",
            platform.name
        )
    })
}
