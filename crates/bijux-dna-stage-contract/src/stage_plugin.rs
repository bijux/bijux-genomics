use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::stage_plan::StagePlanV1;
use bijux_dna_core::contract::{ArtifactRef, StageOperatingMode, StageReportContract};
use bijux_dna_core::metrics::MetricsEnvelope;
use bijux_dna_core::prelude::invariants::{InvariantResultV1, StageVerdictV1};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageInvocationV1 {
    pub command: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub expected_outputs: Vec<ArtifactRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StagePluginOutputV1 {
    pub metrics: MetricsEnvelope<serde_json::Value>,
    pub artifacts: Vec<ArtifactRef>,
    #[serde(default)]
    pub operating_mode: StageOperatingMode,
    #[serde(default)]
    pub report_parts: Vec<StageReportPartV1>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub invariants: Vec<InvariantResultV1>,
    #[serde(default)]
    pub verdict: Option<StageVerdictV1>,
    #[serde(default)]
    pub event_hints: Vec<StageEventHintV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageReportPartV1 {
    pub name: String,
    pub file_name: String,
    pub contract: StageReportContract,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageEventHintV1 {
    pub event_name: String,
    pub status: String,
    pub attrs: serde_json::Value,
}

pub trait StagePlugin {
    fn handles_stage(&self, stage_id: &str) -> bool;
    /// # Errors
    /// Returns an error if the stage invocation cannot be materialized.
    fn materialize(&self, plan: &StagePlanV1) -> Result<StageInvocationV1>;
    /// # Errors
    /// Returns an error if outputs cannot be parsed into metrics/artifacts.
    fn parse_outputs(
        &self,
        plan: &StagePlanV1,
        outputs: &[ArtifactRef],
    ) -> Result<StagePluginOutputV1>;
}

/// # Errors
/// Returns an error if a report payload violates its declared contract.
pub fn validate_report_parts(report_parts: &[StageReportPartV1]) -> Result<()> {
    for report in report_parts {
        if report.contract.report_id.trim().is_empty()
            || report.contract.schema_version.trim().is_empty()
        {
            return Err(anyhow!(
                "report part {} is missing report_id or schema_version",
                report.name
            ));
        }
        for field in &report.contract.required_fields {
            if report.payload.get(field).is_none() {
                return Err(anyhow!(
                    "report part {} is missing required field {} for {}",
                    report.name,
                    field,
                    report.contract.report_id
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_report_parts, StageReportPartV1};

    #[test]
    fn validate_report_parts_rejects_missing_required_fields() {
        let err = validate_report_parts(&[StageReportPartV1 {
            name: "trim_report".to_string(),
            file_name: "trim_report.json".to_string(),
            contract: bijux_dna_core::contract::StageReportContract {
                report_id: "fastq.trim_reads.report".to_string(),
                kind: bijux_dna_core::contract::StageReportKind::Qc,
                schema_version: "bijux.fastq.trim_reads.report.v1".to_string(),
                required_fields: vec!["stage_id".to_string(), "tool_id".to_string()],
                advisory_fields: Vec::new(),
                severity: bijux_dna_core::contract::ReportSeverity::Warning,
            },
            payload: serde_json::json!({"stage_id": "fastq.trim_reads"}),
        }])
        .unwrap_err();

        assert!(err.to_string().contains("missing required field tool_id"));
    }
}
