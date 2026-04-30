use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const SUBSAMPLE_READS_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.subsample_reads.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SubsampleReadsReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: String,
    pub sampling_mode: String,
    pub seed: u64,
    pub target_count: Option<u64>,
    pub target_fraction: Option<f64>,
    pub reads_in_r1: u64,
    pub reads_in_r2: Option<u64>,
    pub reads_out_r1: u64,
    pub reads_out_r2: Option<u64>,
    pub pairs_preserved: bool,
    pub output_r1: String,
    pub output_r2: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{SubsampleReadsReportV1, SUBSAMPLE_READS_REPORT_SCHEMA_VERSION};

    #[test]
    fn subsample_reads_report_round_trips() {
        let report = SubsampleReadsReportV1 {
            schema_version: SUBSAMPLE_READS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.subsample_reads".to_string(),
            stage_id: "fastq.subsample_reads".to_string(),
            tool_id: "bijux".to_string(),
            paired_mode: "paired_end".to_string(),
            sampling_mode: "count".to_string(),
            seed: 7,
            target_count: Some(50),
            target_fraction: None,
            reads_in_r1: 100,
            reads_in_r2: Some(100),
            reads_out_r1: 50,
            reads_out_r2: Some(50),
            pairs_preserved: true,
            output_r1: "subsampled_R1.fastq.gz".to_string(),
            output_r2: Some("subsampled_R2.fastq.gz".to_string()),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: SubsampleReadsReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.reads_out_r1, 50);
        assert!(decoded.pairs_preserved);
    }
}
