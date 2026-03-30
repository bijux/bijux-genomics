use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::{
    qc_post::{QcAggregationEngine, QcAggregationScope},
    PairedMode,
};

pub const REPORT_QC_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.report_qc.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GovernedQcContributorV1 {
    pub contributor_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub artifact_id: String,
    pub artifact_role: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ReportQcReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub aggregation_engine: QcAggregationEngine,
    pub aggregation_scope: QcAggregationScope,
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub mean_q: f64,
    pub contamination_rate: f64,
    #[serde(default)]
    pub adapter_content_max: Option<f64>,
    #[serde(default)]
    pub adapter_content_mean: Option<f64>,
    #[serde(default)]
    pub duplication_rate: Option<f64>,
    #[serde(default)]
    pub n_rate: Option<f64>,
    #[serde(default)]
    pub kmer_warning_count: Option<u64>,
    #[serde(default)]
    pub overrepresented_sequence_count: Option<u64>,
    #[serde(default)]
    pub multiqc_sample_count: Option<u64>,
    #[serde(default)]
    pub multiqc_module_count: Option<u64>,
    #[serde(default)]
    pub raw_fastqc_dir: Option<String>,
    #[serde(default)]
    pub trimmed_fastqc_dir: Option<String>,
    #[serde(default)]
    pub multiqc_report: Option<String>,
    #[serde(default)]
    pub multiqc_data: Option<String>,
    pub governed_qc_input_count: u64,
    pub governed_qc_contributor_stage_ids: Vec<String>,
    pub governed_qc_contributor_tool_ids: Vec<String>,
    pub governed_qc_contributors: Vec<GovernedQcContributorV1>,
    #[serde(default)]
    pub governed_qc_lineage_hash: Option<String>,
    #[serde(default)]
    pub governed_qc_inputs_manifest: Option<String>,
    #[serde(default)]
    pub runtime_s: Option<f64>,
    #[serde(default)]
    pub memory_mb: Option<f64>,
    #[serde(default)]
    pub exit_code: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::{GovernedQcContributorV1, ReportQcReportV1, REPORT_QC_REPORT_SCHEMA_VERSION};
    use crate::params::{
        qc_post::{QcAggregationEngine, QcAggregationScope},
        PairedMode,
    };

    #[test]
    fn report_qc_report_contract_round_trips() {
        let report = ReportQcReportV1 {
            schema_version: REPORT_QC_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.report_qc".to_string(),
            stage_id: "fastq.report_qc".to_string(),
            tool_id: "multiqc".to_string(),
            paired_mode: PairedMode::PairedEnd,
            aggregation_engine: QcAggregationEngine::Multiqc,
            aggregation_scope: QcAggregationScope::GovernedQcArtifacts,
            reads_in: 200,
            reads_out: 200,
            bases_in: 20000,
            bases_out: 20000,
            pairs_in: Some(100),
            pairs_out: Some(100),
            mean_q: 31.2,
            contamination_rate: 0.04,
            adapter_content_max: Some(0.1),
            adapter_content_mean: Some(0.03),
            duplication_rate: Some(0.08),
            n_rate: Some(0.001),
            kmer_warning_count: Some(2),
            overrepresented_sequence_count: Some(1),
            multiqc_sample_count: Some(2),
            multiqc_module_count: Some(5),
            raw_fastqc_dir: Some("raw_fastqc".to_string()),
            trimmed_fastqc_dir: Some("trimmed_fastqc".to_string()),
            multiqc_report: Some("multiqc_report.html".to_string()),
            multiqc_data: Some("multiqc_data".to_string()),
            governed_qc_input_count: 3,
            governed_qc_contributor_stage_ids: vec![
                "fastq.detect_adapters".to_string(),
                "fastq.trim_reads".to_string(),
            ],
            governed_qc_contributor_tool_ids: vec!["fastp".to_string(), "fastqc".to_string()],
            governed_qc_contributors: vec![GovernedQcContributorV1 {
                contributor_id: "fastq.trim_reads.fastp".to_string(),
                stage_id: "fastq.trim_reads".to_string(),
                tool_id: "fastp".to_string(),
                artifact_id: "report_json".to_string(),
                artifact_role: "report_json".to_string(),
                path: "trim/report.json".to_string(),
            }],
            governed_qc_lineage_hash: Some("lineage".to_string()),
            governed_qc_inputs_manifest: Some("governed_qc_inputs_manifest.json".to_string()),
            runtime_s: Some(3.2),
            memory_mb: Some(128.0),
            exit_code: Some(0),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: ReportQcReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "multiqc");
        assert_eq!(decoded.multiqc_sample_count, Some(2));
        assert_eq!(decoded.governed_qc_contributors.len(), 1);
    }
}
