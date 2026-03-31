use std::collections::HashMap;

use anyhow::{anyhow, Result};
use bijux_dna_analyze::{ImageQaOutcome, ImageQaRecord};

use crate::api::{PlatformSpec, ToolImageSpec};
use crate::image_qa::support::resolve_image_for_run;
use crate::image_qa::QaStage;

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
