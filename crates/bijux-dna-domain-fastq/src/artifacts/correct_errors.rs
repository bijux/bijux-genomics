use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::params::{
    correct::{CorrectionEngine, QualityEncoding},
    PairedMode,
};

pub const CORRECT_ERRORS_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.correct_errors.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CorrectErrorsReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub correction_engine: CorrectionEngine,
    pub quality_encoding: QualityEncoding,
    pub kmer_size: Option<u32>,
    pub musket_kmer_budget: Option<u64>,
    pub genome_size: Option<u64>,
    pub max_memory_gb: Option<u32>,
    pub trusted_kmer_artifact: Option<PathBuf>,
    pub conservative_mode: bool,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub output_r1: String,
    pub output_r2: Option<String>,
    pub report_json: String,
    pub corrected_reads: Option<u64>,
    pub changed_reads: Option<u64>,
    pub unchanged_reads: Option<u64>,
    pub reads_in: Option<u64>,
    pub reads_out: Option<u64>,
    pub bases_in: Option<u64>,
    pub bases_out: Option<u64>,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
    pub mean_q_before: Option<f64>,
    pub mean_q_after: Option<f64>,
    pub kmer_fix_rate: Option<f64>,
    pub correction_effect: Option<serde_json::Value>,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i32>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
    pub backend_metrics: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::{CorrectErrorsReportV1, CORRECT_ERRORS_REPORT_SCHEMA_VERSION};
    use crate::params::correct::{CorrectionEngine, QualityEncoding};
    use crate::PairedMode;
    use std::path::PathBuf;

    #[test]
    fn correct_errors_report_contract_round_trips() {
        let report = CorrectErrorsReportV1 {
            schema_version: CORRECT_ERRORS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.correct_errors".to_string(),
            stage_id: "fastq.correct_errors".to_string(),
            tool_id: "lighter".to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 8,
            correction_engine: CorrectionEngine::Lighter,
            quality_encoding: QualityEncoding::Phred33,
            kmer_size: Some(31),
            musket_kmer_budget: None,
            genome_size: Some(3_000_000),
            max_memory_gb: Some(16),
            trusted_kmer_artifact: Some(PathBuf::from("trusted.kmers")),
            conservative_mode: false,
            input_r1: "reads.fastq.gz".to_string(),
            input_r2: None,
            output_r1: "corrected.fastq.gz".to_string(),
            output_r2: None,
            report_json: "correct_report.json".to_string(),
            corrected_reads: Some(100),
            changed_reads: Some(12),
            unchanged_reads: Some(88),
            reads_in: Some(100),
            reads_out: Some(100),
            bases_in: Some(1_000),
            bases_out: Some(980),
            pairs_in: None,
            pairs_out: None,
            mean_q_before: Some(30.0),
            mean_q_after: Some(32.0),
            kmer_fix_rate: Some(0.12),
            correction_effect: Some(serde_json::json!({
                "outputs_changed": true,
                "bases_delta": -20,
                "mean_q_delta": 2.0
            })),
            runtime_s: Some(12.5),
            memory_mb: Some(512.0),
            exit_code: Some(0),
            raw_backend_report: None,
            raw_backend_report_format: None,
            backend_metrics: Some(serde_json::json!({
                "reads_changed": 12
            })),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: CorrectErrorsReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "lighter");
        assert_eq!(decoded.kmer_size, Some(31));
        assert_eq!(decoded.musket_kmer_budget, None);
        assert_eq!(decoded.corrected_reads, Some(100));
        assert_eq!(decoded.changed_reads, Some(12));
        assert_eq!(decoded.unchanged_reads, Some(88));
    }
}
