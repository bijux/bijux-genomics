use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::PairedMode;

/// Schema identity for the local smoke observation envelope.
pub const LOCAL_REMOVE_CHIMERAS_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.remove_chimeras.local_smoke.report.v1";

/// Local smoke observation; planned tool identity and actual report producer remain distinct.
/// This envelope alone does not prove execution of the planned backend.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LocalRemoveChimerasSmokeReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub sample_id: String,
    pub planned_tool_id: String,
    pub report_tool_id: String,
    pub checked_sequence_count: u64,
    pub chimera_count: u64,
    pub non_chimera_count: u64,
    pub filtered_representative_sequences: String,
    pub non_chimeric_fasta: String,
    pub chimeras_tsv: String,
    pub case_report_json: String,
    pub chimera_metrics_json: String,
    pub chimeras_fasta: String,
    pub raw_backend_report: String,
}

pub const REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.remove_chimeras.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RemoveChimerasReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub method: String,
    pub detection_scope: String,
    pub chimera_removed_definition: String,
    pub input_reads: String,
    pub output_reads: String,
    pub chimera_metrics_json: String,
    pub chimeras_fasta: Option<String>,
    pub uchime_report_tsv: Option<String>,
    pub reads_in: Option<u64>,
    pub reads_out: Option<u64>,
    pub chimeras_removed: Option<u64>,
    pub chimera_fraction: Option<f64>,
    pub used_fallback: bool,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i32>,
    pub backend_metrics: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::{RemoveChimerasReportV1, REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION};
    use crate::params::PairedMode;

    #[test]
    fn remove_chimeras_report_contract_round_trips() {
        let report = RemoveChimerasReportV1 {
            schema_version: REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.remove_chimeras".to_string(),
            stage_id: "fastq.remove_chimeras".to_string(),
            tool_id: "vsearch".to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 2,
            method: "vsearch_uchime_denovo".to_string(),
            detection_scope: "denovo".to_string(),
            chimera_removed_definition:
                "reads flagged as de_novo chimeras are excluded from downstream abundance tables"
                    .to_string(),
            input_reads: "merged.fastq.gz".to_string(),
            output_reads: "nonchimeras.fastq.gz".to_string(),
            chimera_metrics_json: "chimera_metrics.json".to_string(),
            chimeras_fasta: Some("chimeras.fasta".to_string()),
            uchime_report_tsv: Some("uchime.tsv".to_string()),
            reads_in: Some(100),
            reads_out: Some(92),
            chimeras_removed: Some(8),
            chimera_fraction: Some(0.08),
            used_fallback: false,
            raw_backend_report: Some("uchime.tsv".to_string()),
            raw_backend_report_format: Some("vsearch_uchime_tsv".to_string()),
            runtime_s: Some(2.1),
            memory_mb: Some(64.0),
            exit_code: Some(0),
            backend_metrics: Some(serde_json::json!({
                "flagged_records": 8_u64,
                "parsed_records": 100_u64,
            })),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: RemoveChimerasReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "vsearch");
        assert_eq!(decoded.threads, 2);
        assert_eq!(decoded.chimera_fraction, Some(0.08));
        assert_eq!(decoded.raw_backend_report_format.as_deref(), Some("vsearch_uchime_tsv"));
    }
}

#[cfg(test)]
mod local_smoke_contract_tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn local_smoke_observation_preserves_wire_shape_and_producer_identity() -> Result<()> {
        let original: serde_json::Value = serde_json::from_str(
            r#"{
    "schema_version": "bijux.fastq.remove_chimeras.local_smoke.report.v1",
    "stage_id": "fastq.remove_chimeras",
    "sample_id": "authored_contract_value",
    "planned_tool_id": "planned_backend",
    "report_tool_id": "local_emitter",
    "checked_sequence_count": 1,
    "chimera_count": 1,
    "non_chimera_count": 1,
    "filtered_representative_sequences": "authored_contract_value",
    "non_chimeric_fasta": "authored_contract_value",
    "chimeras_tsv": "authored_contract_value",
    "case_report_json": "authored_contract_value",
    "chimera_metrics_json": "authored_contract_value",
    "chimeras_fasta": "authored_contract_value",
    "raw_backend_report": "authored_contract_value"
}"#,
        )?;
        let report: LocalRemoveChimerasSmokeReportV1 = serde_json::from_value(original.clone())?;
        assert_eq!(report.schema_version, LOCAL_REMOVE_CHIMERAS_SMOKE_REPORT_SCHEMA_VERSION);
        assert_ne!(report.planned_tool_id, report.report_tool_id);
        assert_eq!(serde_json::to_value(&report)?, original);
        let mut unsupported = original;
        unsupported["unrecognized"] = serde_json::json!(true);
        assert!(serde_json::from_value::<LocalRemoveChimerasSmokeReportV1>(unsupported).is_err());
        let schema = serde_json::to_value(schemars::schema_for!(LocalRemoveChimerasSmokeReportV1))?;
        assert_eq!(schema["additionalProperties"], serde_json::json!(false));
        Ok(())
    }
}
