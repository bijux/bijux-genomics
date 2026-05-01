use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::PairedMode;

pub const DETECT_DUPLICATES_PREMERGE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.detect_duplicates_premerge.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DetectDuplicatesPremergeReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub duplicate_detection_policy: String,
    pub measurement_scope: String,
    pub modifies_reads: bool,
    pub advisory_only: bool,
    pub reads_in: u64,
    pub duplicate_signal_reads: u64,
    pub duplicate_signal_fraction: f64,
    pub compared_read_pairs: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::{
        DetectDuplicatesPremergeReportV1, DETECT_DUPLICATES_PREMERGE_REPORT_SCHEMA_VERSION,
    };
    use crate::params::PairedMode;

    #[test]
    fn detect_duplicates_premerge_report_round_trips() {
        let report = DetectDuplicatesPremergeReportV1 {
            schema_version: DETECT_DUPLICATES_PREMERGE_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.detect_duplicates_premerge".to_string(),
            stage_id: "fastq.detect_duplicates_premerge".to_string(),
            tool_id: "bijux".to_string(),
            paired_mode: PairedMode::PairedEnd,
            duplicate_detection_policy: "report_only".to_string(),
            measurement_scope: "premerge_sequence_signature".to_string(),
            modifies_reads: false,
            advisory_only: true,
            reads_in: 200,
            duplicate_signal_reads: 30,
            duplicate_signal_fraction: 0.15,
            compared_read_pairs: Some(100),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: DetectDuplicatesPremergeReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert!(decoded.advisory_only);
        assert_eq!(decoded.duplicate_signal_fraction, 0.15);
    }
}
