use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::PairedMode;

pub const INFER_ASVS_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.infer_asvs.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InferAsvsReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub denoising_method: String,
    pub pooling_mode: String,
    pub chimera_policy: String,
    pub requires_r_runtime: bool,
    pub output_table_kind: String,
    pub input_reads_r1: String,
    pub input_reads_r2: Option<String>,
    pub asv_table_tsv: String,
    pub asv_sequences_fasta: String,
    pub taxonomy_ready_fasta: String,
    pub taxonomy_ready_fastq: String,
    pub report_json: String,
    pub asv_count: u64,
    pub sample_count: u64,
    pub representative_sequence_count: u64,
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
    use super::{InferAsvsReportV1, INFER_ASVS_REPORT_SCHEMA_VERSION};
    use crate::params::PairedMode;

    #[test]
    fn infer_asvs_report_contract_round_trips() {
        let report = InferAsvsReportV1 {
            schema_version: INFER_ASVS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.infer_asvs".to_string(),
            stage_id: "fastq.infer_asvs".to_string(),
            tool_id: "dada2".to_string(),
            paired_mode: PairedMode::PairedEnd,
            denoising_method: "dada2".to_string(),
            pooling_mode: "independent".to_string(),
            chimera_policy: "remove_bimera_denovo".to_string(),
            requires_r_runtime: true,
            output_table_kind: "asv_abundance_table".to_string(),
            input_reads_r1: "reads_R1.fastq.gz".to_string(),
            input_reads_r2: Some("reads_R2.fastq.gz".to_string()),
            asv_table_tsv: "asv_abundance.tsv".to_string(),
            asv_sequences_fasta: "asv_sequences.fasta".to_string(),
            taxonomy_ready_fasta: "taxonomy_ready.fasta".to_string(),
            taxonomy_ready_fastq: "taxonomy_ready.fastq".to_string(),
            report_json: "infer_asvs_report.json".to_string(),
            asv_count: 18,
            sample_count: 4,
            representative_sequence_count: 18,
            used_fallback: false,
            raw_backend_report: Some("dada2_run_summary.json".to_string()),
            raw_backend_report_format: Some("dada2_run_summary_json".to_string()),
            runtime_s: Some(12.4),
            memory_mb: Some(384.0),
            exit_code: Some(0),
            backend_metrics: Some(serde_json::json!({
                "nonchimera_reads": 1600_u64,
            })),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: InferAsvsReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "dada2");
        assert_eq!(decoded.asv_count, 18);
        assert_eq!(decoded.raw_backend_report_format.as_deref(), Some("dada2_run_summary_json"));
    }
}
