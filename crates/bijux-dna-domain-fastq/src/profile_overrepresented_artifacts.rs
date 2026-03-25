use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::PairedMode;

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
