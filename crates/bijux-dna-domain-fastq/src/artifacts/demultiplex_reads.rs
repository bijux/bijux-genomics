use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const DEMULTIPLEX_READS_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.demultiplex_reads.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DemultiplexSampleSummaryV1 {
    pub sample_id: String,
    pub barcode: String,
    pub reads: u64,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DemultiplexReadsReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub mismatch_policy: String,
    pub max_mismatches: u8,
    pub sample_sheet_validated: bool,
    pub reads_in: u64,
    pub assigned_reads: u64,
    pub undetermined_reads: u64,
    pub undetermined_path: String,
    pub samples: Vec<DemultiplexSampleSummaryV1>,
}

#[cfg(test)]
mod tests {
    use super::{
        DemultiplexReadsReportV1, DemultiplexSampleSummaryV1,
        DEMULTIPLEX_READS_REPORT_SCHEMA_VERSION,
    };

    #[test]
    fn demultiplex_reads_report_round_trips() {
        let report = DemultiplexReadsReportV1 {
            schema_version: DEMULTIPLEX_READS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.demultiplex_reads".to_string(),
            stage_id: "fastq.demultiplex_reads".to_string(),
            tool_id: "bijux".to_string(),
            mismatch_policy: "hamming_prefix".to_string(),
            max_mismatches: 1,
            sample_sheet_validated: true,
            reads_in: 100,
            assigned_reads: 96,
            undetermined_reads: 4,
            undetermined_path: "undetermined.fastq.gz".to_string(),
            samples: vec![DemultiplexSampleSummaryV1 {
                sample_id: "sample-a".to_string(),
                barcode: "ACGT".to_string(),
                reads: 48,
                output_path: "sample-a.fastq.gz".to_string(),
            }],
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: DemultiplexReadsReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.assigned_reads, 96);
        assert_eq!(decoded.undetermined_reads, 4);
    }
}
