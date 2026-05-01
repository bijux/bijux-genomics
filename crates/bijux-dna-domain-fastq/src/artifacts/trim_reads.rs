use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::PrepareAdapterBankReportV1;
use crate::params::PairedMode;

pub const TRIM_READS_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.trim_reads.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TrimReadsReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub trimming_backend: String,
    pub backend_mode: String,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub output_r1: String,
    pub output_r2: Option<String>,
    pub min_length: u32,
    pub quality_cutoff: Option<u32>,
    pub adapter_policy: String,
    pub polyx_policy: Option<String>,
    pub n_policy: Option<String>,
    pub contaminant_policy: Option<String>,
    pub adapter_bank_id: Option<String>,
    pub adapter_bank_hash: Option<String>,
    pub adapter_preset: Option<String>,
    pub detected_adapter_source: Option<String>,
    pub adapter_overrides: Option<serde_json::Value>,
    pub prepared_adapter_bank: Option<PrepareAdapterBankReportV1>,
    pub polyx_bank_id: Option<String>,
    pub polyx_bank_hash: Option<String>,
    pub polyx_preset: Option<String>,
    pub contaminant_bank_id: Option<String>,
    pub contaminant_bank_hash: Option<String>,
    pub contaminant_preset: Option<String>,
    pub reads_in: Option<u64>,
    pub reads_out: Option<u64>,
    pub bases_in: Option<u64>,
    pub bases_out: Option<u64>,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
    pub mean_q_before: Option<f64>,
    pub mean_q_after: Option<f64>,
    pub effective_trim_params: serde_json::Value,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{PrepareAdapterBankReportV1, TrimReadsReportV1, TRIM_READS_REPORT_SCHEMA_VERSION};
    use crate::params::PairedMode;

    #[test]
    fn trim_reads_report_contract_round_trips() {
        let report = TrimReadsReportV1 {
            schema_version: TRIM_READS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.trim_reads".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            tool_id: "fastp".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 4,
            trimming_backend: "fastp".to_string(),
            backend_mode: "enforced".to_string(),
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            output_r1: "trimmed_R1.fastq.gz".to_string(),
            output_r2: Some("trimmed_R2.fastq.gz".to_string()),
            min_length: 30,
            quality_cutoff: Some(20),
            adapter_policy: "bank".to_string(),
            polyx_policy: Some("trim".to_string()),
            n_policy: Some("drop".to_string()),
            contaminant_policy: Some("none".to_string()),
            adapter_bank_id: Some("illumina".to_string()),
            adapter_bank_hash: Some("sha256:adapter".to_string()),
            adapter_preset: Some("default".to_string()),
            detected_adapter_source: Some("fastqc_summary".to_string()),
            adapter_overrides: Some(serde_json::json!({
                "enable": ["AGATCGGAAGAGC"],
                "disable": ["polyA"],
            })),
            prepared_adapter_bank: Some(PrepareAdapterBankReportV1 {
                schema_version: "bijux.fastq.prepare_adapter_bank.report.v1".to_string(),
                stage: "fastq.prepare_adapter_bank".to_string(),
                stage_id: "fastq.prepare_adapter_bank".to_string(),
                tool_id: "bijux".to_string(),
                bank_id: "illumina".to_string(),
                bank_version: "2026.04".to_string(),
                bank_hash: "sha256:adapter".to_string(),
                presets_hash: "sha256:presets".to_string(),
                preset: "default".to_string(),
                preset_hash: "sha256:preset".to_string(),
                enabled_categories: vec!["illumina".to_string()],
                disabled_categories: vec!["polyA".to_string()],
                enable_adapters: vec!["AGATCGGAAGAGC".to_string()],
                disable_adapters: vec!["polyA".to_string()],
                enabled_adapter_ids: vec!["illumina_r1".to_string()],
            }),
            polyx_bank_id: Some("polyx-bank".to_string()),
            polyx_bank_hash: Some("sha256:polyx".to_string()),
            polyx_preset: Some("illumina_twocolor".to_string()),
            contaminant_bank_id: Some("contaminants".to_string()),
            contaminant_bank_hash: Some("sha256:contaminant".to_string()),
            contaminant_preset: Some("illumina_default".to_string()),
            reads_in: Some(100),
            reads_out: Some(90),
            bases_in: Some(10_000),
            bases_out: Some(8_700),
            pairs_in: Some(50),
            pairs_out: Some(45),
            mean_q_before: Some(29.1),
            mean_q_after: Some(31.5),
            effective_trim_params: serde_json::json!({
                "min_length": 30,
                "quality_cutoff": 20,
                "adapter_policy": "bank",
            }),
            runtime_s: Some(12.0),
            memory_mb: Some(256.0),
            raw_backend_report: Some("trim.fastp.json".to_string()),
            raw_backend_report_format: Some("fastp_json".to_string()),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: TrimReadsReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "fastp");
        assert_eq!(decoded.paired_mode, PairedMode::PairedEnd);
        assert_eq!(decoded.threads, 4);
        assert_eq!(decoded.backend_mode, "enforced");
        assert_eq!(decoded.raw_backend_report_format.as_deref(), Some("fastp_json"));
        assert_eq!(
            decoded.adapter_overrides,
            Some(serde_json::json!({
                "enable": ["AGATCGGAAGAGC"],
                "disable": ["polyA"],
            }))
        );
    }
}
