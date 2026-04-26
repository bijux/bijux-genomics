//! Defaults ledger for pipeline parameter/tool provenance.
//! Ordering is canonical (BTreeMap) and JSON serialization must be stable.

use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use bijux_dna_core::ids::{validate_stage_id, validate_tool_id};
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

impl<'de> Deserialize<'de> for DefaultsLedgerV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawDefaultsLedgerV1 {
            pipeline_id: PipelineId,
            tools: BTreeMap<StageId, ToolId>,
            params: BTreeMap<StageId, serde_json::Value>,
            #[serde(default)]
            thresholds: BTreeMap<StageId, serde_json::Value>,
            tool_provenance: BTreeMap<StageId, DefaultProvenanceV1>,
            param_provenance: BTreeMap<StageId, DefaultProvenanceV1>,
            #[serde(default)]
            assumptions: Vec<String>,
            #[serde(default)]
            citations: BTreeMap<String, String>,
        }

        let raw = RawDefaultsLedgerV1::deserialize(deserializer)?;
        let params = raw
            .params
            .into_iter()
            .map(|(stage_id, value)| {
                let params = super::serde_codec::from_stage_json(stage_id.as_str(), value)
                    .map_err(|err| {
                        serde::de::Error::custom(format!(
                            "defaults ledger params for {} could not be parsed: {err}",
                            stage_id.as_str()
                        ))
                    })?;
                Ok((stage_id, params))
            })
            .collect::<std::result::Result<BTreeMap<_, _>, D::Error>>()?;
        Ok(Self {
            pipeline_id: raw.pipeline_id,
            tools: raw.tools,
            params,
            thresholds: raw.thresholds,
            tool_provenance: raw.tool_provenance,
            param_provenance: raw.param_provenance,
            assumptions: raw.assumptions,
            citations: raw.citations,
        })
    }
}

impl DefaultsLedgerV1 {
    /// # Errors
    /// Returns an error when required provenance fields are missing.
    pub fn validate_strict(&self) -> Result<()> {
        for (stage_id, tool_id) in &self.tools {
            validate_stage_id(stage_id)
                .map_err(|err| anyhow!("defaults ledger has invalid tool stage id: {err}"))?;
            validate_tool_id(tool_id)
                .map_err(|err| anyhow!("defaults ledger has invalid tool id: {err}"))?;
        }
        for stage_id in self.params.keys() {
            validate_stage_id(stage_id)
                .map_err(|err| anyhow!("defaults ledger has invalid params stage id: {err}"))?;
        }
        for stage_id in self.tools.keys() {
            if !self.tool_provenance.contains_key(stage_id) {
                return Err(anyhow!(
                    "defaults ledger tool_provenance is missing stage {}",
                    stage_id.as_str()
                ));
            }
        }
        for stage_id in self.params.keys() {
            if !self.param_provenance.contains_key(stage_id) {
                return Err(anyhow!(
                    "defaults ledger param_provenance is missing stage {}",
                    stage_id.as_str()
                ));
            }
        }
        for (stage_id, prov) in &self.tool_provenance {
            if !self.tools.contains_key(stage_id) {
                return Err(anyhow!(
                    "defaults ledger tool_provenance has orphan stage {}",
                    stage_id.as_str()
                ));
            }
            validate_default_provenance(stage_id.as_str(), "tool_provenance", prov)?;
        }
        for (stage_id, prov) in &self.param_provenance {
            if !self.params.contains_key(stage_id) {
                return Err(anyhow!(
                    "defaults ledger param_provenance has orphan stage {}",
                    stage_id.as_str()
                ));
            }
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
