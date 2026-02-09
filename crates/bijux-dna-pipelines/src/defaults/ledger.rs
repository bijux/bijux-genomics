//! Defaults ledger for pipeline parameter/tool provenance.
//! Ordering is canonical (BTreeMap) and JSON serialization must be stable.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::DefaultParams;
use crate::PipelineId;
use bijux_dna_core::ids::{StageId, ToolId};

#[derive(Debug, Clone, Serialize)]
pub struct DefaultProvenanceV1 {
    pub rationale: String,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub comparability_implications: Vec<String>,
    #[serde(default)]
    pub citations: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DefaultsLedgerV1 {
    pub pipeline_id: PipelineId,
    pub tools: BTreeMap<StageId, ToolId>,
    pub params: BTreeMap<StageId, DefaultParams>,
    #[serde(default)]
    pub thresholds: BTreeMap<StageId, serde_json::Value>,
    pub tool_provenance: BTreeMap<StageId, DefaultProvenanceV1>,
    pub param_provenance: BTreeMap<StageId, DefaultProvenanceV1>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub citations: BTreeMap<String, String>,
}
