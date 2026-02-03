use std::collections::BTreeMap;

use serde::Serialize;

use crate::PipelineId;

#[derive(Debug, Clone, Serialize)]
pub struct DefaultsLedgerV1 {
    pub pipeline_id: PipelineId,
    pub tools: BTreeMap<String, String>,
    pub params: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub thresholds: BTreeMap<String, serde_json::Value>,
    pub rationales: BTreeMap<String, String>,
}
