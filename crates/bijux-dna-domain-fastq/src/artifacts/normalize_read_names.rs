use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const NORMALIZE_READ_NAMES_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.normalize_read_names.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NormalizeReadNamesReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: String,
    pub reads_in: u64,
    pub reads_out: u64,
    pub reversible_provenance_embedded: bool,
    pub mate_identity_preserved: bool,
    pub output_r1: String,
    pub output_r2: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{NormalizeReadNamesReportV1, NORMALIZE_READ_NAMES_REPORT_SCHEMA_VERSION};

    #[test]
    fn normalize_read_names_report_round_trips() {
        let report = NormalizeReadNamesReportV1 {
            schema_version: NORMALIZE_READ_NAMES_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.normalize_read_names".to_string(),
            stage_id: "fastq.normalize_read_names".to_string(),
            tool_id: "bijux".to_string(),
            paired_mode: "paired_end".to_string(),
            reads_in: 200,
            reads_out: 200,
            reversible_provenance_embedded: true,
            mate_identity_preserved: true,
            output_r1: "normalized_R1.fastq.gz".to_string(),
            output_r2: Some("normalized_R2.fastq.gz".to_string()),
        };
        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: NormalizeReadNamesReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.reads_out, 200);
        assert!(decoded.reversible_provenance_embedded);
    }
}
