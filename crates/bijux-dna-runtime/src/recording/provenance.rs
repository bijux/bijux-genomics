use std::path::Path;

use anyhow::{Context, Result};

use bijux_dna_core::contract::ExecutionGraph;
use bijux_dna_core::metrics::ToolInvocationV1;

use super::io::write_canonical_json;

/// # Errors
/// Returns an error if the provenance file cannot be written.
pub fn write_scientific_provenance(
    run_dir: &Path,
    provenance: &bijux_dna_core::contract::ScientificProvenanceV1,
) -> Result<std::path::PathBuf> {
    let path = run_dir.join("scientific_provenance.json");
    write_canonical_json(&path, provenance).context("write scientific_provenance.json")?;
    Ok(path)
}

/// Build and write a minimal scientific provenance file derived from the plan.
///
/// This is intended for contract tests and dry-run validation.
/// # Errors
/// Returns an error if provenance serialization or writing fails.
pub fn write_plan_provenance(run_dir: &Path, plan: &ExecutionGraph) -> Result<std::path::PathBuf> {
    let mut invocations = Vec::new();
    let mut params_hashes = std::collections::BTreeMap::new();
    for step in plan.steps() {
        let params = serde_json::json!({ "command": step.command.template });
        let key = format!("{}:{}", step.step_id.0, step.image.image);
        let hash = bijux_dna_core::prelude::hashing::params_hash(&params)?;
        params_hashes.insert(key, hash);
        let image_digest = step
            .image
            .digest
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let params_provenance = serde_json::json!({
            "tool_params": params.clone(),
            "defaults": serde_json::json!({}),
            "overrides": serde_json::json!({}),
            "effective_params": params.clone(),
        });
        let params_provenance_normalized =
            bijux_dna_core::contract::canonical::canonicalize_json_value(&params_provenance);
        invocations.push(ToolInvocationV1 {
            schema_version: "bijux.tool_invocation.v1".to_string(),
            contract_version: bijux_dna_core::contract::ContractVersion::v1(),
            stage_id: step.stage_id.clone(),
            tool_id: bijux_dna_core::ids::ToolId::from_static("unknown"),
            tool_version: "unknown".to_string(),
            resolved_tool_version: None,
            image_digest,
            runner_kind: "fake".to_string(),
            platform: "unknown".to_string(),
            parameters_json: params.clone(),
            parameters_json_normalized: params.clone(),
            effective_params_json: params.clone(),
            effective_params_json_normalized: params.clone(),
            params_provenance,
            params_provenance_normalized,
            adapter_bank: None,
            banks: None,
            bank_assets: None,
            resources: step.resources.clone(),
            environment: std::collections::BTreeMap::new(),
            input_hashes: Vec::new(),
            output_hashes: Vec::new(),
            executed_command: None,
        });
    }
    let provenance = crate::provenance::build_scientific_provenance(
        plan.pipeline_id().to_string(),
        plan.planner_version().to_string(),
        &params_hashes,
        &invocations,
    );
    write_scientific_provenance(run_dir, &provenance)
}
