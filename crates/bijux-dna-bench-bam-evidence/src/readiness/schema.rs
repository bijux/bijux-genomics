use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
#[derive(Debug, Clone)]
pub(super) struct BamStageSchemaContract {
    pub extension_id: String,
    pub required_keys: Vec<String>,
}
pub(in crate::readiness) fn collect_bam_stage_schema_contracts(
    schema: &serde_json::Value,
    stages: impl IntoIterator<Item = &'static str>,
) -> Result<BTreeMap<String, BamStageSchemaContract>> {
    let stage_defs = schema
        .get("$defs")
        .and_then(|value| value.get("stages"))
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| anyhow!("BAM normalized metrics schema is missing object `$defs.stages`"))?;

    let mut contracts = BTreeMap::new();
    for stage_id in stages {
        let stage_contract = stage_defs
            .get(stage_id)
            .ok_or_else(|| anyhow!("BAM normalized metrics schema is missing `{stage_id}`"))?
            .get("allOf")
            .and_then(serde_json::Value::as_array)
            .and_then(|items| items.get(1))
            .ok_or_else(|| {
                anyhow!("BAM normalized metrics stage `{stage_id}` is missing stage extension")
            })?;
        let extension_id = stage_contract
            .get("x-bijux-extension-id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                anyhow!(
                    "BAM normalized metrics stage `{stage_id}` is missing string `x-bijux-extension-id`"
                )
            })?;
        let required_keys = stage_contract
            .get("required")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| {
                anyhow!("BAM normalized metrics stage `{stage_id}` is missing `required` keys")
            })?
            .iter()
            .map(|value| {
                value.as_str().map(str::to_string).ok_or_else(|| {
                    anyhow!("BAM normalized metrics stage `{stage_id}` has non-string required key")
                })
            })
            .collect::<Result<Vec<_>>>()?;
        contracts.insert(
            stage_id.to_string(),
            BamStageSchemaContract { extension_id: extension_id.to_string(), required_keys },
        );
    }

    Ok(contracts)
}
