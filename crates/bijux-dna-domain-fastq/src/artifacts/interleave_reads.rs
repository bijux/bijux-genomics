use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const INTERLEAVE_READS_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.interleave_reads.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InterleaveReadsReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub reads_in_r1: u64,
    pub reads_in_r2: u64,
    pub interleaved_pairs: u64,
    pub output_reads: u64,
    pub pairing_validated: bool,
    pub output_interleaved: String,
}

#[cfg(test)]
mod tests {
    use super::{InterleaveReadsReportV1, INTERLEAVE_READS_REPORT_SCHEMA_VERSION};

    #[test]
    fn interleave_reads_report_round_trips() {
        let report = InterleaveReadsReportV1 {
            schema_version: INTERLEAVE_READS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.interleave_reads".to_string(),
            stage_id: "fastq.interleave_reads".to_string(),
            tool_id: "bijux".to_string(),
            reads_in_r1: 10,
            reads_in_r2: 10,
            interleaved_pairs: 10,
            output_reads: 20,
            pairing_validated: true,
            output_interleaved: "interleaved.fastq.gz".to_string(),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: InterleaveReadsReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.output_reads, 20);
        assert!(decoded.pairing_validated);
    }
}
