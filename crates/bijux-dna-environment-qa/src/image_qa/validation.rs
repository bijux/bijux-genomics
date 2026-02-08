use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};

use crate::api::{PlatformSpec, ToolImageSpec};

use super::support::resolve_image_for_run;

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
    if std::env::var("BIJUX_SKIP_QA").is_ok() {
        return Ok(());
    }
    let cwd = std::env::current_dir().map_err(|err| anyhow!("failed to resolve cwd: {err}"))?;
    let qa_sqlite = super::support::image_qa_sqlite_path(&cwd, &platform.name);
    if !qa_sqlite.exists() {
        return Err(anyhow!(
            "image QA results missing; run `bijux image-qa --platform {}`",
            platform.name
        ));
    }
    let conn = bijux_dna_analyze::open_sqlite(&qa_sqlite).context("open image qa sqlite")?;
    let runner = platform.runner.to_string();
    let mut expected_inputs = bijux_dna_analyze::load::sqlite::reports::image_qa_inputs(
        &conn,
        stage,
        &platform.name,
        &runner,
    )?;
    if expected_inputs.is_empty() {
        expected_inputs =
            bijux_dna_analyze::load::sqlite::reports::image_qa_input_hashes_from_records(
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
