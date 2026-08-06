use std::collections::HashMap;

use anyhow::{anyhow, Result};
use bijux_dna_analyze::load::sqlite::reports::image_qa_passed;

use crate::api::{PlatformSpec, ToolImageSpec};
use crate::image_qa::support::resolve_image_for_run;
use crate::image_qa::QaStage;

pub(crate) fn qa_already_passed(
    conn: &rusqlite::Connection,
    stage: QaStage,
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    input_hash: &str,
) -> Result<bool> {
    let stage_id = stage.stage_id();
    let spec = catalog.get(tool).ok_or_else(|| anyhow!("tool {tool} missing from images.toml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    let image_digest =
        spec.digest.as_ref().map_or_else(|| image.full_name.clone(), ToString::to_string);
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
