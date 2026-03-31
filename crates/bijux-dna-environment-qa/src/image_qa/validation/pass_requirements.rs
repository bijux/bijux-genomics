use std::collections::HashMap;

use anyhow::{anyhow, Result};

use crate::api::{PlatformSpec, ToolImageSpec};
use crate::image_qa::support::resolve_image_for_run;

pub(crate) fn ensure_image_qa_inputs_passed<S: ::std::hash::BuildHasher>(
    stage: &str,
    tools: &[String],
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec, S>,
    expected_inputs: &[String],
) -> Result<()> {
    let cwd = std::env::current_dir().map_err(|err| anyhow!("failed to resolve cwd: {err}"))?;
    let qa_sqlite = crate::image_qa::support::image_qa_sqlite_path(&cwd, &platform.name);
    let conn = bijux_dna_analyze::open_sqlite(&qa_sqlite)?;
    let runner = platform.runner.to_string();

    for tool in tools {
        let spec = catalog
            .get(tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from images.toml"))?;
        let image = resolve_image_for_run(spec, platform)?;
        let image_digest = spec
            .digest
            .as_ref()
            .map_or_else(|| image.full_name.clone(), ToString::to_string);
        for input_hash in expected_inputs {
            let passed = bijux_dna_analyze::load::sqlite::reports::image_qa_passed(
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
