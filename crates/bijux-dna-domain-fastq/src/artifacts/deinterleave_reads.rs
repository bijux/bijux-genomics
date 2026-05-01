use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const DEINTERLEAVE_READS_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.deinterleave_reads.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeinterleaveReadsReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub reads_in: u64,
    pub output_pairs: u64,
    pub output_r1_reads: u64,
    pub output_r2_reads: u64,
    pub rejected_records: u64,
    pub output_r1: String,
    pub output_r2: String,
    pub rejected_path: String,
}

#[cfg(test)]
mod tests {
    use super::{DeinterleaveReadsReportV1, DEINTERLEAVE_READS_REPORT_SCHEMA_VERSION};

    #[test]
    fn deinterleave_reads_report_round_trips() {
        let report = DeinterleaveReadsReportV1 {
            schema_version: DEINTERLEAVE_READS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.deinterleave_reads".to_string(),
            stage_id: "fastq.deinterleave_reads".to_string(),
            tool_id: "bijux".to_string(),
            reads_in: 20,
            output_pairs: 9,
            output_r1_reads: 9,
            output_r2_reads: 9,
            rejected_records: 2,
            output_r1: "deinterleaved_R1.fastq.gz".to_string(),
            output_r2: "deinterleaved_R2.fastq.gz".to_string(),
            rejected_path: "deinterleaved_rejected.fastq.gz".to_string(),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: DeinterleaveReadsReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.output_pairs, 9);
        assert_eq!(decoded.rejected_records, 2);
    }
}
