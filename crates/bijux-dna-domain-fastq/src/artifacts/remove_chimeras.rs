use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::PairedMode;

pub const REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.remove_chimeras.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RemoveChimerasReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub method: String,
    pub detection_scope: String,
    pub chimera_removed_definition: String,
    pub input_reads: String,
    pub output_reads: String,
    pub chimera_metrics_json: String,
    pub chimeras_fasta: Option<String>,
    pub uchime_report_tsv: Option<String>,
    pub reads_in: Option<u64>,
    pub reads_out: Option<u64>,
    pub chimeras_removed: Option<u64>,
    pub chimera_fraction: Option<f64>,
    pub used_fallback: bool,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i32>,
    pub backend_metrics: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::{RemoveChimerasReportV1, REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION};
    use crate::params::PairedMode;

    #[test]
    fn remove_chimeras_report_contract_round_trips() {
        let report = RemoveChimerasReportV1 {
            schema_version: REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.remove_chimeras".to_string(),
            stage_id: "fastq.remove_chimeras".to_string(),
            tool_id: "vsearch".to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 2,
            method: "vsearch_uchime_denovo".to_string(),
            detection_scope: "denovo".to_string(),
            chimera_removed_definition:
                "reads flagged as de_novo chimeras are excluded from downstream abundance tables"
                    .to_string(),
            input_reads: "merged.fastq.gz".to_string(),
            output_reads: "nonchimeras.fastq.gz".to_string(),
            chimera_metrics_json: "chimera_metrics.json".to_string(),
            chimeras_fasta: Some("chimeras.fasta".to_string()),
            uchime_report_tsv: Some("uchime.tsv".to_string()),
            reads_in: Some(100),
            reads_out: Some(92),
            chimeras_removed: Some(8),
            chimera_fraction: Some(0.08),
            used_fallback: false,
            raw_backend_report: Some("uchime.tsv".to_string()),
            raw_backend_report_format: Some("vsearch_uchime_tsv".to_string()),
            runtime_s: Some(2.1),
            memory_mb: Some(64.0),
            exit_code: Some(0),
            backend_metrics: Some(serde_json::json!({
                "flagged_records": 8_u64,
                "parsed_records": 100_u64,
            })),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: RemoveChimerasReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "vsearch");
        assert_eq!(decoded.threads, 2);
        assert_eq!(decoded.chimera_fraction, Some(0.08));
        assert_eq!(
            decoded.raw_backend_report_format.as_deref(),
            Some("vsearch_uchime_tsv")
        );
    }
}
