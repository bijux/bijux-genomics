use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const DETECT_INSTRUMENT_ARTIFACTS_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.detect_instrument_artifacts.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DetectInstrumentArtifactsReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: String,
    pub advisory_only: bool,
    pub reads_in: u64,
    pub poly_g_reads: u64,
    pub poly_n_reads: u64,
    pub quality_tail_reads: u64,
    pub patterned_flowcell_suspects: u64,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        DetectInstrumentArtifactsReportV1, DETECT_INSTRUMENT_ARTIFACTS_REPORT_SCHEMA_VERSION,
    };

    #[test]
    fn detect_instrument_artifacts_report_round_trips() {
        let report = DetectInstrumentArtifactsReportV1 {
            schema_version: DETECT_INSTRUMENT_ARTIFACTS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.detect_instrument_artifacts".to_string(),
            stage_id: "fastq.detect_instrument_artifacts".to_string(),
            tool_id: "bijux".to_string(),
            paired_mode: "single_end".to_string(),
            advisory_only: true,
            reads_in: 100,
            poly_g_reads: 20,
            poly_n_reads: 5,
            quality_tail_reads: 8,
            patterned_flowcell_suspects: 3,
            warnings: vec!["poly_g_fraction_high".to_string()],
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: DetectInstrumentArtifactsReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert!(decoded.advisory_only);
        assert_eq!(decoded.poly_g_reads, 20);
    }
}
