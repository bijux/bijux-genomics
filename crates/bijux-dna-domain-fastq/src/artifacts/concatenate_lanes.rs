use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONCATENATE_LANES_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.concatenate_lanes.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConcatenateLaneSummaryV1 {
    pub lane_id: String,
    pub reads_r1: u64,
    pub reads_r2: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConcatenateLanesReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: String,
    pub lanes_concatenated: u64,
    pub reads_in_r1: u64,
    pub reads_in_r2: Option<u64>,
    pub duplicate_read_ids_detected: u64,
    pub duplicate_read_id_examples: Vec<String>,
    pub output_r1_reads: u64,
    pub output_r2_reads: Option<u64>,
    pub output_r1: String,
    pub output_r2: Option<String>,
    pub lanes: Vec<ConcatenateLaneSummaryV1>,
}

#[cfg(test)]
mod tests {
    use super::{
        ConcatenateLaneSummaryV1, ConcatenateLanesReportV1, CONCATENATE_LANES_REPORT_SCHEMA_VERSION,
    };

    #[test]
    fn concatenate_lanes_report_round_trips() {
        let report = ConcatenateLanesReportV1 {
            schema_version: CONCATENATE_LANES_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.concatenate_lanes".to_string(),
            stage_id: "fastq.concatenate_lanes".to_string(),
            tool_id: "bijux".to_string(),
            paired_mode: "paired_end".to_string(),
            lanes_concatenated: 2,
            reads_in_r1: 200,
            reads_in_r2: Some(200),
            duplicate_read_ids_detected: 3,
            duplicate_read_id_examples: vec!["read-1".to_string()],
            output_r1_reads: 200,
            output_r2_reads: Some(200),
            output_r1: "merged_R1.fastq.gz".to_string(),
            output_r2: Some("merged_R2.fastq.gz".to_string()),
            lanes: vec![ConcatenateLaneSummaryV1 {
                lane_id: "L001".to_string(),
                reads_r1: 100,
                reads_r2: Some(100),
            }],
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: ConcatenateLanesReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.lanes_concatenated, 2);
        assert_eq!(decoded.duplicate_read_ids_detected, 3);
    }
}
