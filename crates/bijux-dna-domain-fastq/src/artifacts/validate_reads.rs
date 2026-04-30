use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::{
    validate::{PairSyncPolicy, ValidationMode},
    PairedMode,
};

pub const VALIDATION_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.validate.report.v1";
pub const VALIDATED_READS_MANIFEST_SCHEMA_VERSION: &str = "bijux.fastq.validate.lineage.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidateFailureClass {
    None,
    UnsupportedCompression,
    EmptyInput,
    MalformedRecord,
    InvalidQualityEncoding,
    ValidatorError,
    PairCountMismatch,
    HeaderSyncMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ValidationReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub validation_mode: ValidationMode,
    pub pair_sync_policy: PairSyncPolicy,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub validation_log_r1: String,
    pub validation_log_r2: Option<String>,
    pub validated_inputs: u64,
    pub validated_reads_r1: u64,
    pub validated_reads_r2: Option<u64>,
    pub validated_pairs: Option<u64>,
    pub status_r1: i32,
    pub status_r2: i32,
    pub pair_sync_checked: bool,
    pub pair_sync_pass: Option<bool>,
    pub pair_count_match: Option<bool>,
    pub failure_class: ValidateFailureClass,
    pub strict_pass: bool,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ValidatedReadsManifestV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub validation_mode: ValidationMode,
    pub pair_sync_policy: PairSyncPolicy,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub validation_report: String,
    pub paired_mode: PairedMode,
    pub validated_stream_ids: Vec<String>,
    pub pair_sync_checked: bool,
    pub pair_sync_pass: Option<bool>,
    pub validated_pairs: Option<u64>,
}

impl ValidationReportV1 {
    #[must_use]
    pub fn is_pair_failure(&self) -> bool {
        matches!(
            self.failure_class,
            ValidateFailureClass::PairCountMismatch | ValidateFailureClass::HeaderSyncMismatch
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PairSyncPolicy, PairedMode, ValidateFailureClass, ValidatedReadsManifestV1, ValidationMode,
        ValidationReportV1, VALIDATED_READS_MANIFEST_SCHEMA_VERSION,
        VALIDATION_REPORT_SCHEMA_VERSION,
    };

    #[test]
    fn validation_report_contract_round_trips_with_pair_failure_taxonomy() {
        let report = ValidationReportV1 {
            schema_version: VALIDATION_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.validate_reads".to_string(),
            stage_id: "fastq.validate_reads".to_string(),
            tool_id: "fastqvalidator".to_string(),
            validation_mode: ValidationMode::Strict,
            pair_sync_policy: PairSyncPolicy::RequireHeaderSync,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            validation_log_r1: "validation_r1.log".to_string(),
            validation_log_r2: Some("validation_r2.log".to_string()),
            validated_inputs: 2,
            validated_reads_r1: 101,
            validated_reads_r2: Some(100),
            validated_pairs: Some(100),
            status_r1: 0,
            status_r2: 0,
            pair_sync_checked: true,
            pair_sync_pass: Some(false),
            pair_count_match: Some(false),
            failure_class: ValidateFailureClass::PairCountMismatch,
            strict_pass: false,
            exit_code: 96,
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: ValidationReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert!(decoded.is_pair_failure());
        assert_eq!(decoded.failure_class, ValidateFailureClass::PairCountMismatch);
    }

    #[test]
    fn validated_reads_manifest_contract_round_trips() {
        let manifest = ValidatedReadsManifestV1 {
            schema_version: VALIDATED_READS_MANIFEST_SCHEMA_VERSION.to_string(),
            stage_id: "fastq.validate_reads".to_string(),
            tool_id: "seqtk".to_string(),
            validation_mode: ValidationMode::ReportOnly,
            pair_sync_policy: PairSyncPolicy::SkipHeaderSync,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            validation_report: "validation.json".to_string(),
            paired_mode: PairedMode::PairedEnd,
            validated_stream_ids: vec!["reads_r1".to_string(), "reads_r2".to_string()],
            pair_sync_checked: false,
            pair_sync_pass: None,
            validated_pairs: Some(100),
        };

        let encoded = serde_json::to_string(&manifest)
            .unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: ValidatedReadsManifestV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.paired_mode, PairedMode::PairedEnd);
        assert_eq!(decoded.validated_stream_ids.len(), 2);
    }
}
