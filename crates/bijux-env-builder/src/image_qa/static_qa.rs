use std::collections::HashMap;

use anyhow::{anyhow, Result};

use crate::api::{docker_image_exists, PlatformSpec, ToolImageSpec};

use super::support::resolve_image_for_run;

/// Run static QA checks (image exists, resolves, entrypoint accessible).
pub(crate) fn run_static_qa(
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
) -> Result<()> {
    let spec = catalog
        .get(tool)
        .ok_or_else(|| anyhow!("tool {tool} missing from images.toml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    if !docker_image_exists(&image) {
        return Err(anyhow!(
            "docker image missing for {tool}: {}",
            image.full_name
        ));
    }
    Ok(())
}
