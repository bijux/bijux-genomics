use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::PairedMode;

pub const FILTER_READS_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.filter_reads.report.v3";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FilterReadsReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub output_r1: String,
    pub output_r2: Option<String>,
    pub report_json: String,
    pub max_n: Option<u32>,
    pub max_n_fraction: Option<f64>,
    pub max_n_count: Option<u32>,
    pub low_complexity_threshold: Option<f64>,
    pub entropy_threshold: Option<f64>,
    pub n_policy: Option<String>,
    pub polyx_policy: Option<String>,
    pub contaminant_db: Option<String>,
    pub reads_in: u64,
    pub reads_out: u64,
    pub reads_dropped: u64,
    pub reads_removed_by_n: u64,
    pub reads_removed_by_entropy: u64,
    pub reads_removed_low_complexity: u64,
    pub reads_removed_by_kmer: u64,
    pub reads_removed_contaminant_kmer: u64,
    pub reads_removed_by_length: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i32>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
    pub backend_metrics: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::{FilterReadsReportV1, FILTER_READS_REPORT_SCHEMA_VERSION};
    use crate::params::PairedMode;

    #[test]
    fn filter_reads_report_contract_round_trips() {
        let report = FilterReadsReportV1 {
            schema_version: FILTER_READS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.filter_reads".to_string(),
            stage_id: "fastq.filter_reads".to_string(),
            tool_id: "fastp".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 4,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            output_r1: "filtered_R1.fastq.gz".to_string(),
            output_r2: Some("filtered_R2.fastq.gz".to_string()),
            report_json: "filter_report.json".to_string(),
            max_n: Some(0),
            max_n_fraction: None,
            max_n_count: Some(0),
            low_complexity_threshold: Some(20.0),
            entropy_threshold: Some(20.0),
            n_policy: Some("drop".to_string()),
            polyx_policy: Some("trim".to_string()),
            contaminant_db: Some("contaminants.fa".to_string()),
            reads_in: 100,
            reads_out: 95,
            reads_dropped: 5,
            reads_removed_by_n: 2,
            reads_removed_by_entropy: 1,
            reads_removed_low_complexity: 1,
            reads_removed_by_kmer: 1,
            reads_removed_contaminant_kmer: 1,
            reads_removed_by_length: 0,
            bases_in: 10_000,
            bases_out: 9_200,
            pairs_in: Some(50),
            pairs_out: Some(47),
            mean_q_before: 28.0,
            mean_q_after: 30.0,
            runtime_s: Some(4.2),
            memory_mb: Some(128.0),
            exit_code: Some(0),
            raw_backend_report: Some("fastp.json".to_string()),
            raw_backend_report_format: Some("fastp_json".to_string()),
            backend_metrics: Some(serde_json::json!({
                "passed_filter_reads": 95_u64,
                "too_many_n_reads": 2_u64,
            })),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: FilterReadsReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "fastp");
        assert_eq!(decoded.reads_removed_by_n, 2);
        assert_eq!(
            decoded.raw_backend_report_format.as_deref(),
            Some("fastp_json")
        );
    }
}
