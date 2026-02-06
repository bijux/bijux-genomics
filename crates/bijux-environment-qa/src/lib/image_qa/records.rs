use std::collections::HashMap;

use anyhow::{anyhow, Result};
use bijux_analyze::{image_qa_passed, ImageQaOutcome, ImageQaRecord};

use crate::api::{PlatformSpec, ToolImageSpec};

use super::support::resolve_image_for_run;
use super::QaStage;

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
