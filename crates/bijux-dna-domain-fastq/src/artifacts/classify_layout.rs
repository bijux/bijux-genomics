use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CLASSIFY_LAYOUT_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.classify_layout.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FastqLayoutClassV1 {
    SingleEnd,
    PairedEnd,
    Interleaved,
    Merged,
    Singleton,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ClassifyLayoutReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub layout: FastqLayoutClassV1,
    pub confidence: f64,
    pub files_examined: u32,
    pub records_examined: u64,
    pub pair_sync_observed: Option<bool>,
    pub reasons: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        ClassifyLayoutReportV1, FastqLayoutClassV1, CLASSIFY_LAYOUT_REPORT_SCHEMA_VERSION,
    };

    #[test]
    fn classify_layout_report_round_trips() {
        let report = ClassifyLayoutReportV1 {
            schema_version: CLASSIFY_LAYOUT_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.classify_layout".to_string(),
            stage_id: "fastq.classify_layout".to_string(),
            tool_id: "bijux".to_string(),
            layout: FastqLayoutClassV1::Interleaved,
            confidence: 0.91,
            files_examined: 1,
            records_examined: 128,
            pair_sync_observed: Some(true),
            reasons: vec!["alternating mate tags".to_string()],
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: ClassifyLayoutReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.layout, FastqLayoutClassV1::Interleaved);
        assert_eq!(decoded.files_examined, 1);
    }
}
