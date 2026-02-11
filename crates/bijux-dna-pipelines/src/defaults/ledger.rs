//! Defaults ledger for pipeline parameter/tool provenance.
//! Ordering is canonical (BTreeMap) and JSON serialization must be stable.

use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::DefaultParams;
use crate::PipelineId;
use bijux_dna_core::ids::{StageId, ToolId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultProvenanceV1 {
    pub rationale: String,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub comparability_implications: Vec<String>,
    #[serde(default)]
    pub citations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl DefaultsLedgerV1 {
    /// # Errors
    /// Returns an error when required provenance fields are missing.
    pub fn validate_strict(&self) -> Result<()> {
        for (stage_id, prov) in &self.tool_provenance {
            validate_default_provenance(stage_id.as_str(), "tool_provenance", prov)?;
        }
        for (stage_id, prov) in &self.param_provenance {
            validate_default_provenance(stage_id.as_str(), "param_provenance", prov)?;
        }
        Ok(())
    }
}

fn validate_default_provenance(
    stage_id: &str,
    section: &str,
    provenance: &DefaultProvenanceV1,
) -> Result<()> {
    if provenance.rationale.trim().is_empty()
        || provenance.assumptions.is_empty()
        || provenance.comparability_implications.is_empty()
        || provenance.citations.is_empty()
    {
        return Err(anyhow!(
            "defaults ledger {section} for stage {stage_id} is missing rationale/assumptions/comparability_implications/citations"
        ));
    }
    Ok(())
}
