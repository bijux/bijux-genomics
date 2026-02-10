use std::collections::BTreeSet;

use anyhow::{Context, Result};
use bijux_dna_environment::api::run_smoke_script_batch;
use bijux_dna_runtime::manifests::load_manifests;

/// Run Apptainer QA using the registry-driven smoke contract.
///
/// # Errors
/// Returns an error if manifests cannot be loaded or any smoke contract fails.
pub(crate) fn run_apptainer_image_qa() -> Result<()> {
    let manifests = load_manifests(&std::env::current_dir()?.join("domain"))
        .map_err(|err| anyhow::anyhow!("manifest validation failed: {err}"))?;
    let mut tools = BTreeSet::<String>::new();
    for stage_id in manifests.stages().keys() {
        for tool in manifests.tools_for_stage(stage_id) {
            tools.insert(tool.tool_id.to_string());
        }
    }
    let tool_list = tools.into_iter().collect::<Vec<_>>();
    run_smoke_script_batch("apptainer", &tool_list, "contract")
        .context("run apptainer smoke contract")?;
    Ok(())
}
