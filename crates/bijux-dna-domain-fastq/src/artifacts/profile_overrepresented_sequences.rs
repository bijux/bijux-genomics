use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::PairedMode;

/// Schema identity for the local smoke observation envelope.
pub const LOCAL_PROFILE_OVERREPRESENTED_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.profile_overrepresented_sequences.local_smoke.report.v1";

/// Local smoke observation; planned tool identity and actual report producer remain distinct.
/// This envelope alone does not prove execution of the planned backend.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LocalProfileOverrepresentedSequencesSmokeReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub sample_id: String,
    pub planned_tool_id: String,
    pub report_tool_id: String,
    pub top_sequence: String,
    pub top_count: u64,
    pub top_fraction: f64,
    pub flagged_sequences: u64,
    pub overrepresented_tsv: String,
    pub case_report_json: String,
    pub raw_backend_report: String,
}

pub const PROFILE_OVERREPRESENTED_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.profile_overrepresented.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OverrepresentedSequenceRowV1 {
    pub sequence: String,
    pub count: u64,
    pub fraction: f64,
    pub flag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProfileOverrepresentedReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub top_k: u32,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub overrepresented_sequences_tsv: String,
    pub overrepresented_sequences_json: String,
    pub report_json: String,
    pub sequence_count: u64,
    pub flagged_sequences: u64,
    pub top_fraction: f64,
    pub rows: Vec<OverrepresentedSequenceRowV1>,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i32>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        OverrepresentedSequenceRowV1, ProfileOverrepresentedReportV1,
        PROFILE_OVERREPRESENTED_REPORT_SCHEMA_VERSION,
    };
    use crate::params::PairedMode;

    #[test]
    fn profile_overrepresented_report_contract_round_trips() {
        let report = ProfileOverrepresentedReportV1 {
            schema_version: PROFILE_OVERREPRESENTED_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.profile_overrepresented_sequences".to_string(),
            stage_id: "fastq.profile_overrepresented_sequences".to_string(),
            tool_id: "fastqc".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 4,
            top_k: 25,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            overrepresented_sequences_tsv: "overrepresented_sequences.tsv".to_string(),
            overrepresented_sequences_json: "overrepresented_sequences.json".to_string(),
            report_json: "overrepresented_report.json".to_string(),
            sequence_count: 25,
            flagged_sequences: 3,
            top_fraction: 0.12,
            rows: vec![OverrepresentedSequenceRowV1 {
                sequence: "ACGT".to_string(),
                count: 12,
                fraction: 0.12,
                flag: "overrepresented".to_string(),
            }],
            runtime_s: Some(1.4),
            memory_mb: Some(48.0),
            exit_code: Some(0),
            raw_backend_report: Some("fastqc_data.txt".to_string()),
            raw_backend_report_format: Some("fastqc_module_txt".to_string()),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: ProfileOverrepresentedReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "fastqc");
        assert_eq!(decoded.rows.len(), 1);
        assert_eq!(decoded.top_k, 25);
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
    "schema_version": "bijux.fastq.profile_overrepresented_sequences.local_smoke.report.v1",
    "stage_id": "fastq.profile_overrepresented_sequences",
    "sample_id": "authored_contract_value",
    "planned_tool_id": "planned_backend",
    "report_tool_id": "local_emitter",
    "top_sequence": "authored_contract_value",
    "top_count": 1,
    "top_fraction": 0.5,
    "flagged_sequences": 1,
    "overrepresented_tsv": "authored_contract_value",
    "case_report_json": "authored_contract_value",
    "raw_backend_report": "authored_contract_value"
}"#,
        )?;
        let report: LocalProfileOverrepresentedSequencesSmokeReportV1 =
            serde_json::from_value(original.clone())?;
        assert_eq!(
            report.schema_version,
            LOCAL_PROFILE_OVERREPRESENTED_SMOKE_REPORT_SCHEMA_VERSION
        );
        assert_ne!(report.planned_tool_id, report.report_tool_id);
        assert_eq!(serde_json::to_value(&report)?, original);
        let mut unsupported = original;
        unsupported["unrecognized"] = serde_json::json!(true);
        assert!(serde_json::from_value::<LocalProfileOverrepresentedSequencesSmokeReportV1>(
            unsupported
        )
        .is_err());
        let schema = serde_json::to_value(schemars::schema_for!(
            LocalProfileOverrepresentedSequencesSmokeReportV1
        ))?;
        assert_eq!(schema["additionalProperties"], serde_json::json!(false));
        Ok(())
    }
}
