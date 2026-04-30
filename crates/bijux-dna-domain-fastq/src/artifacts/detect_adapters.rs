use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::{
    detect_adapters::{AdapterEvidenceFormat, AdapterEvidenceScope, AdapterInspectionMode},
    PairedMode,
};

pub const DETECT_ADAPTERS_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.detect_adapters.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DetectAdaptersReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub inspection_mode: AdapterInspectionMode,
    pub report_only: bool,
    pub evidence_engine: String,
    pub evidence_scope: AdapterEvidenceScope,
    pub evidence_format: AdapterEvidenceFormat,
    pub evidence_artifact_id: String,
    pub detected_adapter_source: String,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub report_json: String,
    pub adapter_evidence_dir: String,
    pub recommended_adapter_bank_id: Option<String>,
    pub recommended_adapter_bank_hash: Option<String>,
    pub recommended_adapter_preset: Option<String>,
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
    pub mean_q: f64,
    pub candidate_adapter_count: u64,
    pub adapter_trimmed_fraction: Option<f64>,
    pub adapter_content_max: Option<f64>,
    pub adapter_content_mean: Option<f64>,
    pub duplication_rate: Option<f64>,
    pub n_rate: Option<f64>,
    pub kmer_warning_count: Option<u64>,
    pub overrepresented_sequence_count: Option<u64>,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i32>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{DetectAdaptersReportV1, DETECT_ADAPTERS_REPORT_SCHEMA_VERSION};
    use crate::params::{
        detect_adapters::{AdapterEvidenceFormat, AdapterEvidenceScope, AdapterInspectionMode},
        PairedMode,
    };

    #[test]
    fn detect_adapters_report_contract_round_trips() {
        let report = DetectAdaptersReportV1 {
            schema_version: DETECT_ADAPTERS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.detect_adapters".to_string(),
            stage_id: "fastq.detect_adapters".to_string(),
            tool_id: "fastqc".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 4,
            inspection_mode: AdapterInspectionMode::EvidenceOnly,
            report_only: true,
            evidence_engine: "fastqc".to_string(),
            evidence_scope: AdapterEvidenceScope::FullInput,
            evidence_format: AdapterEvidenceFormat::FastqcSummary,
            evidence_artifact_id: "report_json".to_string(),
            detected_adapter_source: "fastqc_summary".to_string(),
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            report_json: "adapter_report.json".to_string(),
            adapter_evidence_dir: "fastqc".to_string(),
            recommended_adapter_bank_id: Some("illumina".to_string()),
            recommended_adapter_bank_hash: Some("sha256:adapter-bank".to_string()),
            recommended_adapter_preset: Some("illumina-default".to_string()),
            reads_in: 200,
            reads_out: 200,
            bases_in: 20_000,
            bases_out: 20_000,
            pairs_in: Some(100),
            pairs_out: Some(100),
            mean_q: 31.2,
            candidate_adapter_count: 2,
            adapter_trimmed_fraction: Some(0.08),
            adapter_content_max: Some(12.5),
            adapter_content_mean: Some(3.2),
            duplication_rate: Some(0.15),
            n_rate: Some(0.001),
            kmer_warning_count: Some(4),
            overrepresented_sequence_count: Some(3),
            runtime_s: Some(4.0),
            memory_mb: Some(64.0),
            exit_code: Some(0),
            raw_backend_report: Some("fastqc/fastqc_data.txt".to_string()),
            raw_backend_report_format: Some("fastqc_data_txt".to_string()),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: DetectAdaptersReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "fastqc");
        assert_eq!(decoded.evidence_scope, AdapterEvidenceScope::FullInput);
        assert_eq!(decoded.candidate_adapter_count, 2);
        assert_eq!(decoded.detected_adapter_source, "fastqc_summary");
    }
}
