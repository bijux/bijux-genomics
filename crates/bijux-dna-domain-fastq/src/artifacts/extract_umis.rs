use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::{
    umi::{
        UmiDedupPolicy, UmiDownstreamPropagation, UmiExtractionLocation, UmiFailedExtractionPolicy,
        UmiGroupingPolicy, UmiReadNameTransform,
    },
    PairedMode,
};

pub const EXTRACT_UMIS_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.extract_umis.report.v2";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UmiExtractionSummaryV1 {
    pub tag_header_format: String,
    pub extracted_umi_count: u64,
    pub invalid_umi_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExtractUmisReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub umi_pattern: String,
    pub extraction_location: UmiExtractionLocation,
    pub read_name_transform: UmiReadNameTransform,
    pub failed_extraction_policy: UmiFailedExtractionPolicy,
    pub grouping_policy: UmiGroupingPolicy,
    pub downstream_dedup_policy: UmiDedupPolicy,
    pub downstream_propagation: UmiDownstreamPropagation,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub output_r1: String,
    pub output_r2: Option<String>,
    pub report_json: String,
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
    pub reads_with_umi: u64,
    pub failed_extractions: Option<u64>,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i32>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
    pub backend_metrics: Option<serde_json::Value>,
}

impl ExtractUmisReportV1 {
    #[must_use]
    pub fn canonical_umi_summary(&self) -> UmiExtractionSummaryV1 {
        UmiExtractionSummaryV1 {
            tag_header_format: umi_read_name_transform_literal(&self.read_name_transform)
                .to_string(),
            extracted_umi_count: self.reads_with_umi,
            invalid_umi_count: self.failed_extractions.unwrap_or(0),
        }
    }
}

fn umi_read_name_transform_literal(transform: &UmiReadNameTransform) -> &'static str {
    match transform {
        UmiReadNameTransform::AppendToHeader => "append_to_header",
        UmiReadNameTransform::ReplaceHeader => "replace_header",
        UmiReadNameTransform::None => "none",
    }
}

#[cfg(test)]
mod tests {
    use super::{ExtractUmisReportV1, UmiExtractionSummaryV1, EXTRACT_UMIS_REPORT_SCHEMA_VERSION};
    use crate::params::{
        umi::{
            UmiDedupPolicy, UmiDownstreamPropagation, UmiExtractionLocation,
            UmiFailedExtractionPolicy, UmiGroupingPolicy, UmiReadNameTransform,
        },
        PairedMode,
    };

    #[test]
    fn extract_umis_report_contract_round_trips() {
        let report = ExtractUmisReportV1 {
            schema_version: EXTRACT_UMIS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.extract_umis".to_string(),
            stage_id: "fastq.extract_umis".to_string(),
            tool_id: "umi_tools".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 2,
            umi_pattern: "NNNNNNNN".to_string(),
            extraction_location: UmiExtractionLocation::Read1Prefix,
            read_name_transform: UmiReadNameTransform::AppendToHeader,
            failed_extraction_policy: UmiFailedExtractionPolicy::RefuseStage,
            grouping_policy: UmiGroupingPolicy::PairAware,
            downstream_dedup_policy: UmiDedupPolicy::SequenceIdentityRecommended,
            downstream_propagation: UmiDownstreamPropagation::HeaderAndReport,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            output_r1: "umi_reads_R1.fastq.gz".to_string(),
            output_r2: Some("umi_reads_R2.fastq.gz".to_string()),
            report_json: "umi_report.json".to_string(),
            reads_in: 200,
            reads_out: 200,
            bases_in: 20_000,
            bases_out: 20_000,
            pairs_in: Some(100),
            pairs_out: Some(100),
            reads_with_umi: 200,
            failed_extractions: Some(0),
            mean_q_before: 30.0,
            mean_q_after: 30.0,
            runtime_s: Some(1.4),
            memory_mb: Some(64.0),
            exit_code: Some(0),
            raw_backend_report: Some("umi_tools.extract.log".to_string()),
            raw_backend_report_format: Some("umi_tools_log".to_string()),
            backend_metrics: Some(serde_json::json!({
                "reads_with_umi_fraction": 1.0,
            })),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: ExtractUmisReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "umi_tools");
        assert_eq!(decoded.umi_pattern, "NNNNNNNN");
        assert_eq!(decoded.reads_with_umi, 200);
        assert_eq!(decoded.read_name_transform, UmiReadNameTransform::AppendToHeader);
        assert_eq!(decoded.grouping_policy, UmiGroupingPolicy::PairAware);
    }

    #[test]
    fn extract_umis_report_derives_canonical_umi_summary() {
        let report = ExtractUmisReportV1 {
            schema_version: EXTRACT_UMIS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.extract_umis".to_string(),
            stage_id: "fastq.extract_umis".to_string(),
            tool_id: "umi_tools".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 2,
            umi_pattern: "NNNNNNNN".to_string(),
            extraction_location: UmiExtractionLocation::Read1Prefix,
            read_name_transform: UmiReadNameTransform::AppendToHeader,
            failed_extraction_policy: UmiFailedExtractionPolicy::RetainUnmodified,
            grouping_policy: UmiGroupingPolicy::PairAware,
            downstream_dedup_policy: UmiDedupPolicy::SequenceIdentityRecommended,
            downstream_propagation: UmiDownstreamPropagation::HeaderAndReport,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            output_r1: "umi_reads_R1.fastq.gz".to_string(),
            output_r2: Some("umi_reads_R2.fastq.gz".to_string()),
            report_json: "umi_report.json".to_string(),
            reads_in: 200,
            reads_out: 196,
            bases_in: 20_000,
            bases_out: 19_600,
            pairs_in: Some(100),
            pairs_out: Some(98),
            reads_with_umi: 196,
            failed_extractions: Some(4),
            mean_q_before: 30.0,
            mean_q_after: 30.0,
            runtime_s: Some(1.4),
            memory_mb: Some(64.0),
            exit_code: Some(0),
            raw_backend_report: None,
            raw_backend_report_format: None,
            backend_metrics: None,
        };

        assert_eq!(
            report.canonical_umi_summary(),
            UmiExtractionSummaryV1 {
                tag_header_format: "append_to_header".to_string(),
                extracted_umi_count: 196,
                invalid_umi_count: 4,
            }
        );
    }
}
