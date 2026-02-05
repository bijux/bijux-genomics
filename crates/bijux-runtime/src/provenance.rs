use std::collections::BTreeMap;

use bijux_core::contract::{ScientificProvenanceV1, ToolProvenanceV1};
use bijux_core::metrics::ToolInvocationV1;

#[must_use]
pub fn build_scientific_provenance(
    pipeline_id: String,
    planner_version: String,
    params_hashes: &BTreeMap<String, String>,
    invocations: &[ToolInvocationV1],
) -> ScientificProvenanceV1 {
    let mut tools = Vec::new();
    let mut input_hashes = Vec::new();
    let mut reference_hashes = BTreeMap::new();
    for invocation in invocations {
        input_hashes.extend(invocation.input_hashes.clone());
        let key = format!("{}:{}", invocation.stage_id, invocation.tool_id);
        let params_hash = params_hashes
            .get(&key)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        tools.push(ToolProvenanceV1 {
            stage_id: invocation.stage_id.clone(),
            tool_id: invocation.tool_id.clone(),
            tool_version: invocation.tool_version.clone(),
            image_digest: invocation.image_digest.clone(),
            params_hash,
            parameters_json: invocation.parameters_json.clone(),
            input_hashes: invocation.input_hashes.clone(),
            output_hashes: invocation.output_hashes.clone(),
            banks: invocation.banks.clone(),
            bank_assets: invocation.bank_assets.clone(),
        });
        if let Some(banks) = &invocation.banks {
            if let Some(obj) = banks.as_object() {
                for (name, entry) in obj {
                    if let Some(hash) = entry.get("bank_hash").and_then(|v| v.as_str()) {
                        reference_hashes.insert(format!("bank:{name}"), hash.to_string());
                    }
                }
            }
        }
    }
    input_hashes.sort();
    input_hashes.dedup();
    tools.sort_by(|a, b| a.stage_id.cmp(&b.stage_id).then(a.tool_id.cmp(&b.tool_id)));
    ScientificProvenanceV1 {
        schema_version: "bijux.scientific_provenance.v1".to_string(),
        pipeline_id,
        planner_version,
        tools,
        input_hashes,
        reference_hashes,
    }
}
