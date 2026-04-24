use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::PairedMode;

pub const NORMALIZE_PRIMERS_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.normalize_primers.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NormalizePrimersReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub primer_set_id: String,
    pub marker_id: Option<String>,
    pub primer_fasta: Option<String>,
    pub orientation_policy: String,
    pub max_mismatch_rate: f64,
    pub min_overlap_bp: u32,
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
    pub primer_trimmed_reads: Option<u64>,
    pub primer_trimmed_fraction: Option<f64>,
    pub orientation_forward_fraction: Option<f64>,
    pub primer_orientation_report: String,
    pub primer_stats_json: String,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub used_fallback: bool,
    pub backend_metrics: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::{NormalizePrimersReportV1, NORMALIZE_PRIMERS_REPORT_SCHEMA_VERSION};
    use crate::params::PairedMode;

    #[test]
    fn normalize_primers_report_contract_round_trips() {
        let report = NormalizePrimersReportV1 {
            schema_version: NORMALIZE_PRIMERS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.normalize_primers".to_string(),
            stage_id: "fastq.normalize_primers".to_string(),
            tool_id: "cutadapt".to_string(),
            paired_mode: PairedMode::PairedEnd,
            primer_set_id: "16S_universal_v1".to_string(),
            marker_id: Some("16S".to_string()),
            primer_fasta: Some("assets/reference/primers/16S_universal_v1.fasta".to_string()),
            orientation_policy: "normalize_to_forward_primer".to_string(),
            max_mismatch_rate: 0.10,
            min_overlap_bp: 10,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            output_r1: "normalized_R1.fastq.gz".to_string(),
            output_r2: Some("normalized_R2.fastq.gz".to_string()),
            reads_in: Some(200),
            reads_out: Some(200),
            bases_in: Some(10_000),
            bases_out: Some(9_600),
            pairs_in: Some(100),
            pairs_out: Some(100),
            primer_trimmed_reads: Some(190),
            primer_trimmed_fraction: Some(0.95),
            orientation_forward_fraction: Some(0.94),
            primer_orientation_report: "primer_orientation.tsv".to_string(),
            primer_stats_json: "primer_stats.json".to_string(),
            raw_backend_report: Some("primer_stats.json".to_string()),
            raw_backend_report_format: Some("cutadapt_json".to_string()),
            runtime_s: Some(3.1),
            memory_mb: Some(96.0),
            used_fallback: false,
            backend_metrics: Some(serde_json::json!({
                "tool": "cutadapt",
                "trimmed_reads": 190_u64,
            })),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: NormalizePrimersReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "cutadapt");
        assert_eq!(decoded.primer_set_id, "16S_universal_v1");
        assert_eq!(decoded.primer_trimmed_reads, Some(190));
        assert_eq!(decoded.raw_backend_report_format.as_deref(), Some("cutadapt_json"));
    }
}
