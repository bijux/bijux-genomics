use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::PairedMode;

pub const PROFILE_READ_LENGTHS_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.profile_read_lengths.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProfileReadLengthBinV1 {
    pub read_length: u64,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProfileReadLengthsReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub histogram_bins: u32,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub length_distribution_tsv: String,
    pub length_distribution_json: String,
    pub report_json: String,
    pub read_count: u64,
    #[serde(default)]
    pub min_read_length: u64,
    pub mean_read_length: f64,
    #[serde(default)]
    pub median_read_length: f64,
    pub max_read_length: u64,
    pub distinct_lengths: u64,
    pub histogram: Vec<ProfileReadLengthBinV1>,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i32>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        ProfileReadLengthBinV1, ProfileReadLengthsReportV1,
        PROFILE_READ_LENGTHS_REPORT_SCHEMA_VERSION,
    };
    use crate::params::PairedMode;

    #[test]
    fn profile_read_lengths_report_contract_round_trips() {
        let report = ProfileReadLengthsReportV1 {
            schema_version: PROFILE_READ_LENGTHS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.profile_read_lengths".to_string(),
            stage_id: "fastq.profile_read_lengths".to_string(),
            tool_id: "seqkit_stats".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 2,
            histogram_bins: 64,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            length_distribution_tsv: "length_distribution.tsv".to_string(),
            length_distribution_json: "length_distribution.json".to_string(),
            report_json: "profile_read_lengths_report.json".to_string(),
            read_count: 200,
            min_read_length: 90,
            mean_read_length: 101.2,
            median_read_length: 100.0,
            max_read_length: 150,
            distinct_lengths: 12,
            histogram: vec![ProfileReadLengthBinV1 { read_length: 100, count: 180 }],
            runtime_s: Some(1.1),
            memory_mb: Some(16.0),
            exit_code: Some(0),
            raw_backend_report: Some("length_distribution.tsv".to_string()),
            raw_backend_report_format: Some("seqkit_fx2tab_tsv".to_string()),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: ProfileReadLengthsReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "seqkit_stats");
        assert_eq!(decoded.threads, 2);
        assert_eq!(decoded.histogram_bins, 64);
        assert_eq!(decoded.min_read_length, 90);
        assert_eq!(decoded.median_read_length, 100.0);
        assert_eq!(decoded.histogram.len(), 1);
    }
}
