use std::collections::HashMap;

use anyhow::{anyhow, Result};
use bijux_core::prelude::{CommandSpecV1, ContainerImageRefV1, ToolExecutionSpecV1};
use bijux_environment::api::{PlatformSpec, ToolImageSpec};

use crate::backend::docker::executor::resolve_image_for_run;

pub fn build_tool_execution_spec<S: ::std::hash::BuildHasher>(
    stage_id: &str,
    tool_id: &str,
    registry: &bijux_core::contract::ToolRegistry,
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
) -> Result<ToolExecutionSpecV1> {
    let stage_id = bijux_core::ids::StageId::try_from(stage_id).map_err(|err| anyhow!("{err}"))?;
    let tool_id = bijux_core::ids::ToolId::try_from(tool_id).map_err(|err| anyhow!("{err}"))?;
    let manifest = registry
        .tool_by_id(&stage_id, &tool_id)
        .ok_or_else(|| anyhow!("tool {tool_id} missing from manifest for {stage_id}"))?;
    let spec = catalog
        .get(tool_id.as_str())
        .ok_or_else(|| anyhow!("tool {tool_id} missing from images.toml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    Ok(ToolExecutionSpecV1 {
        tool_id: tool_id.clone(),
        tool_version: spec.version.clone(),
        image: ContainerImageRefV1 {
            image: image.full_name,
            digest: spec.digest.clone(),
        },
        command: CommandSpecV1 {
            template: manifest.command_template.clone(),
        },
        resources: manifest.constraints.clone(),
    })
}
