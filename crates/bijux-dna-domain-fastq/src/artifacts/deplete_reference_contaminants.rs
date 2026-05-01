use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::PairedMode;

pub const DEPLETE_REFERENCE_CONTAMINANTS_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.deplete_reference_contaminants.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DepleteReferenceContaminantsReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub reference_catalog_id: String,
    pub contaminant_reference: String,
    pub reference_index_artifact_id: String,
    pub reference_index_backend: String,
    pub reference_build_id: Option<String>,
    pub reference_digest: Option<String>,
    pub match_threshold: Option<f64>,
    pub retained_read_role: String,
    pub rejected_read_role: String,
    pub retain_unmapped_pairs: bool,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub output_r1: String,
    pub output_r2: Option<String>,
    pub report_json: String,
    pub reads_in: u64,
    pub reads_out: u64,
    pub reads_removed: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub bases_removed: u64,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
    pub contaminant_fraction_removed: f64,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i32>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
    pub backend_metrics: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::{
        DepleteReferenceContaminantsReportV1, DEPLETE_REFERENCE_CONTAMINANTS_REPORT_SCHEMA_VERSION,
    };
    use crate::params::PairedMode;

    #[test]
    fn deplete_reference_contaminants_report_contract_round_trips() {
        let report = DepleteReferenceContaminantsReportV1 {
            schema_version: DEPLETE_REFERENCE_CONTAMINANTS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.deplete_reference_contaminants".to_string(),
            stage_id: "fastq.deplete_reference_contaminants".to_string(),
            tool_id: "bowtie2".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 8,
            reference_catalog_id: "contaminant_reference".to_string(),
            contaminant_reference: "phix_and_spikeins".to_string(),
            reference_index_artifact_id: "reference_index".to_string(),
            reference_index_backend: "bowtie2_build".to_string(),
            reference_build_id: Some("2026.03".to_string()),
            reference_digest: Some("sha256:example".to_string()),
            match_threshold: Some(0.95),
            retained_read_role: "contaminant_screened_reads".to_string(),
            rejected_read_role: "removed_contaminant_reads".to_string(),
            retain_unmapped_pairs: true,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            output_r1: "contaminant_screened_R1.fastq.gz".to_string(),
            output_r2: Some("contaminant_screened_R2.fastq.gz".to_string()),
            report_json: "contaminant_screen_report.json".to_string(),
            reads_in: 200,
            reads_out: 160,
            reads_removed: 40,
            bases_in: 20_000,
            bases_out: 15_600,
            bases_removed: 4_400,
            pairs_in: Some(100),
            pairs_out: Some(80),
            contaminant_fraction_removed: 0.2,
            runtime_s: Some(9.8),
            memory_mb: Some(512.0),
            exit_code: Some(0),
            raw_backend_report: Some("bowtie2.contaminant.metrics.txt".to_string()),
            raw_backend_report_format: Some("bowtie2_met_file".to_string()),
            backend_metrics: Some(serde_json::json!({
                "reads_removed": 40_u64,
            })),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: DepleteReferenceContaminantsReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "bowtie2");
        assert_eq!(decoded.reads_removed, 40);
        assert_eq!(decoded.match_threshold, Some(0.95));
        assert_eq!(decoded.raw_backend_report_format.as_deref(), Some("bowtie2_met_file"));
    }
}
