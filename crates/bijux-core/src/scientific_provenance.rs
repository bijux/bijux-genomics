use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ToolInvocationV1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolProvenanceV1 {
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: String,
    pub params_hash: String,
    pub parameters_json: serde_json::Value,
    pub input_hashes: Vec<String>,
    pub output_hashes: Vec<String>,
    #[serde(default)]
    pub banks: Option<serde_json::Value>,
    #[serde(default)]
    pub bank_assets: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScientificProvenanceV1 {
    pub schema_version: String,
    pub pipeline_id: String,
    pub planner_version: String,
    pub tools: Vec<ToolProvenanceV1>,
    pub input_hashes: Vec<String>,
    pub reference_hashes: BTreeMap<String, String>,
}

impl ScientificProvenanceV1 {
    #[must_use]
    pub fn from_invocations(
        pipeline_id: String,
        planner_version: String,
        params_hashes: &BTreeMap<String, String>,
        invocations: &[ToolInvocationV1],
    ) -> Self {
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
                            reference_hashes.insert(
                                format!("bank:{name}"),
                                hash.to_string(),
                            );
                        }
                    }
                }
            }
        }
        input_hashes.sort();
        input_hashes.dedup();
        tools.sort_by(|a, b| a.stage_id.cmp(&b.stage_id).then(a.tool_id.cmp(&b.tool_id)));
        Self {
            schema_version: "bijux.scientific_provenance.v1".to_string(),
            pipeline_id,
            planner_version,
            tools,
            input_hashes,
            reference_hashes,
        }
    }
}
