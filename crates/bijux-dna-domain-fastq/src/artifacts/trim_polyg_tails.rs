use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::PairedMode;

pub const TRIM_POLYG_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.trim_polyg_tails.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TrimPolygReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub trim_polyg: bool,
    pub min_polyg_run: u32,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub output_r1: String,
    pub output_r2: Option<String>,
    pub reads_in: Option<u64>,
    pub reads_out: Option<u64>,
    pub bases_in: Option<u64>,
    pub bases_out: Option<u64>,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
    pub mean_q_before: Option<f64>,
    pub mean_q_after: Option<f64>,
    pub trimmed_tail_count: Option<u64>,
    pub bases_trimmed_polyg: Option<u64>,
    pub polyx_bank_id: Option<String>,
    pub polyx_bank_hash: Option<String>,
    pub polyx_preset: Option<String>,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
    pub backend_metrics: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::{TrimPolygReportV1, TRIM_POLYG_REPORT_SCHEMA_VERSION};
    use crate::params::PairedMode;

    #[test]
    fn trim_polyg_report_contract_round_trips() {
        let report = TrimPolygReportV1 {
            schema_version: TRIM_POLYG_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.trim_polyg_tails".to_string(),
            stage_id: "fastq.trim_polyg_tails".to_string(),
            tool_id: "fastp".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 4,
            trim_polyg: true,
            min_polyg_run: 10,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            output_r1: "trimmed_R1.fastq.gz".to_string(),
            output_r2: Some("trimmed_R2.fastq.gz".to_string()),
            reads_in: Some(100),
            reads_out: Some(98),
            bases_in: Some(10_000),
            bases_out: Some(9_820),
            pairs_in: Some(50),
            pairs_out: Some(49),
            mean_q_before: Some(27.9),
            mean_q_after: Some(28.4),
            trimmed_tail_count: Some(12),
            bases_trimmed_polyg: Some(180),
            polyx_bank_id: Some("polyx".to_string()),
            polyx_bank_hash: Some("sha256:polyx".to_string()),
            polyx_preset: Some("illumina_twocolor".to_string()),
            runtime_s: Some(4.2),
            memory_mb: Some(96.0),
            raw_backend_report: Some("trim_polyg.fastp.json".to_string()),
            raw_backend_report_format: Some("fastp_json".to_string()),
            backend_metrics: Some(serde_json::json!({
                "schema_version": "bijux.fastp.metrics.v1",
                "passed_filter_reads": 98_u64,
            })),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: TrimPolygReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "fastp");
        assert_eq!(decoded.paired_mode, PairedMode::PairedEnd);
        assert_eq!(decoded.threads, 4);
        assert_eq!(decoded.trimmed_tail_count, Some(12));
        assert_eq!(decoded.bases_trimmed_polyg, Some(180));
        assert_eq!(decoded.raw_backend_report_format.as_deref(), Some("fastp_json"));
    }
}
