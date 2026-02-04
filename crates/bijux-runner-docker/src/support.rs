use std::collections::HashMap;

use anyhow::{anyhow, Result};
use bijux_core::{CommandSpecV1, ContainerImageRefV1, ToolExecutionSpecV1, ToolId};
use bijux_env_runtime::api::{PlatformSpec, ToolImageSpec};

use crate::executor::resolve_image_for_run;

pub fn build_tool_execution_spec<S: ::std::hash::BuildHasher>(
    stage_id: &str,
    tool_id: &str,
    registry: &bijux_core::ToolRegistry,
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
) -> Result<ToolExecutionSpecV1> {
    let manifest = registry
        .tool_by_id(stage_id, tool_id)
        .ok_or_else(|| anyhow!("tool {tool_id} missing from manifest for {stage_id}"))?;
    let spec = catalog
        .get(tool_id)
        .ok_or_else(|| anyhow!("tool {tool_id} missing from images.toml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    Ok(ToolExecutionSpecV1 {
        tool_id: ToolId(tool_id.to_string()),
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
