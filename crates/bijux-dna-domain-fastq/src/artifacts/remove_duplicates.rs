use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::remove_duplicates::DedupMode;
use crate::params::PairedMode;

pub const REMOVE_DUPLICATES_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.remove_duplicates.report.v2";
pub const REMOVE_DUPLICATES_PROVENANCE_SCHEMA_VERSION: &str =
    "bijux.fastq.remove_duplicates.provenance.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DuplicateClassEntryV1 {
    pub class: String,
    pub reads_removed: u64,
    pub paired_mode: PairedMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RemoveDuplicatesProvenanceV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub dedup_mode: DedupMode,
    pub keep_order: bool,
    pub duplicates_removed: u64,
    pub dedup_rate: f64,
    pub backend_log: Option<String>,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub output_r1: String,
    pub output_r2: Option<String>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RemoveDuplicatesReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub dedup_mode: DedupMode,
    pub keep_order: bool,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub output_r1: String,
    pub output_r2: Option<String>,
    pub reads_in: u64,
    pub reads_out: u64,
    pub reads_in_r2: Option<u64>,
    pub reads_out_r2: Option<u64>,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
    pub pair_count_match: Option<bool>,
    pub duplicates_removed: u64,
    pub dedup_rate: f64,
    pub duplicate_classes_tsv: Option<String>,
    pub duplicate_provenance_json: Option<String>,
    #[serde(default)]
    pub duplicate_classes: Vec<DuplicateClassEntryV1>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::{
        DuplicateClassEntryV1, RemoveDuplicatesProvenanceV1, RemoveDuplicatesReportV1,
        REMOVE_DUPLICATES_PROVENANCE_SCHEMA_VERSION, REMOVE_DUPLICATES_REPORT_SCHEMA_VERSION,
    };
    use crate::params::remove_duplicates::DedupMode;
    use crate::params::PairedMode;

    #[test]
    fn remove_duplicates_report_contract_round_trips() {
        let report = RemoveDuplicatesReportV1 {
            schema_version: REMOVE_DUPLICATES_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.remove_duplicates".to_string(),
            stage_id: "fastq.remove_duplicates".to_string(),
            tool_id: "clumpify".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 4,
            dedup_mode: DedupMode::OpticalAware,
            keep_order: false,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            output_r1: "dedup_R1.fastq.gz".to_string(),
            output_r2: Some("dedup_R2.fastq.gz".to_string()),
            reads_in: 200,
            reads_out: 180,
            reads_in_r2: Some(200),
            reads_out_r2: Some(180),
            pairs_in: Some(200),
            pairs_out: Some(180),
            pair_count_match: Some(true),
            duplicates_removed: 20,
            dedup_rate: 0.1,
            duplicate_classes_tsv: Some("duplicate_classes.tsv".to_string()),
            duplicate_provenance_json: Some("duplicate_provenance.json".to_string()),
            duplicate_classes: vec![DuplicateClassEntryV1 {
                class: "duplicate".to_string(),
                reads_removed: 20,
                paired_mode: PairedMode::PairedEnd,
            }],
            raw_backend_report: Some("clumpify.log".to_string()),
            raw_backend_report_format: Some("clumpify_log".to_string()),
            runtime_s: Some(4.5),
            memory_mb: Some(128.0),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: RemoveDuplicatesReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "clumpify");
        assert_eq!(decoded.threads, 4);
        assert_eq!(decoded.dedup_mode, DedupMode::OpticalAware);
        assert_eq!(decoded.duplicate_classes.len(), 1);
    }

    #[test]
    fn remove_duplicates_provenance_contract_round_trips() {
        let provenance = RemoveDuplicatesProvenanceV1 {
            schema_version: REMOVE_DUPLICATES_PROVENANCE_SCHEMA_VERSION.to_string(),
            stage_id: "fastq.remove_duplicates".to_string(),
            tool_id: "fastuniq".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 1,
            dedup_mode: DedupMode::Exact,
            keep_order: true,
            duplicates_removed: 12,
            dedup_rate: 0.06,
            backend_log: Some("fastuniq.log".to_string()),
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            output_r1: "dedup_R1.fastq.gz".to_string(),
            output_r2: Some("dedup_R2.fastq.gz".to_string()),
            raw_backend_report: Some("fastuniq.log".to_string()),
            raw_backend_report_format: Some("fastuniq_log".to_string()),
        };

        let encoded = serde_json::to_string(&provenance)
            .unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: RemoveDuplicatesProvenanceV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "fastuniq");
        assert_eq!(decoded.threads, 1);
        assert_eq!(decoded.dedup_mode, DedupMode::Exact);
        assert_eq!(decoded.raw_backend_report_format.as_deref(), Some("fastuniq_log"));
    }
}
