use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::{merge::MergeEngine, merge::UnmergedReadPolicy, PairedMode};

/// Schema identity for the local smoke observation envelope.
pub const LOCAL_MERGE_PAIRS_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.merge_pairs.local_smoke.report.v1";

/// Local smoke observation; planned tool identity and actual report producer remain distinct.
/// This envelope alone does not prove execution of the planned backend.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LocalMergePairsSmokeReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub sample_id: String,
    pub planned_tool_id: String,
    pub report_tool_id: String,
    pub merge_overlap: u32,
    pub min_length: u32,
    pub input_pair_count: u64,
    pub merged_count: u64,
    pub unmerged_r1_count: u64,
    pub unmerged_r2_count: u64,
    pub discarded_count: u64,
    pub merged_fastq_gz: String,
    pub unmerged_r1_fastq_gz: Option<String>,
    pub unmerged_r2_fastq_gz: Option<String>,
    pub case_report_json: String,
    pub raw_backend_report: Option<String>,
}

pub const MERGE_PAIRS_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.merge_pairs.report.v2";

#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MergePairCountsV1 {
    pub input_pair_count: u64,
    pub merged_pair_count: u64,
    pub unmerged_pair_count: u64,
    pub discarded_pair_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MergePairsReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub merge_engine: MergeEngine,
    pub threads: u32,
    pub merge_overlap: Option<u32>,
    pub min_len: Option<u32>,
    pub unmerged_read_policy: UnmergedReadPolicy,
    pub input_r1: String,
    pub input_r2: String,
    pub merged_reads: String,
    pub unmerged_reads_r1: Option<String>,
    pub unmerged_reads_r2: Option<String>,
    pub reads_r1: u64,
    pub reads_r2: u64,
    pub reads_merged: u64,
    pub reads_unmerged: u64,
    pub merge_rate: f64,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
}

impl MergePairsReportV1 {
    #[must_use]
    pub fn canonical_pair_counts(&self) -> MergePairCountsV1 {
        let input_pair_count = self.reads_r1.min(self.reads_r2);
        let merged_pair_count = self.reads_merged.min(input_pair_count);
        let unmerged_pair_count =
            self.reads_unmerged.min(input_pair_count.saturating_sub(merged_pair_count));
        let discarded_pair_count =
            input_pair_count.saturating_sub(merged_pair_count + unmerged_pair_count);
        MergePairCountsV1 {
            input_pair_count,
            merged_pair_count,
            unmerged_pair_count,
            discarded_pair_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MergePairCountsV1, MergePairsReportV1, MERGE_PAIRS_REPORT_SCHEMA_VERSION};
    use crate::params::merge::{MergeEngine, UnmergedReadPolicy};
    use crate::params::PairedMode;

    #[test]
    fn merge_pairs_report_contract_round_trips() {
        let report = MergePairsReportV1 {
            schema_version: MERGE_PAIRS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.merge_pairs".to_string(),
            stage_id: "fastq.merge_pairs".to_string(),
            tool_id: "pear".to_string(),
            paired_mode: PairedMode::PairedEnd,
            merge_engine: MergeEngine::Pear,
            threads: 4,
            merge_overlap: Some(20),
            min_len: Some(80),
            unmerged_read_policy: UnmergedReadPolicy::EmitUnmergedPairs,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: "reads_R2.fastq.gz".to_string(),
            merged_reads: "merged.fastq.gz".to_string(),
            unmerged_reads_r1: Some("unmerged_R1.fastq.gz".to_string()),
            unmerged_reads_r2: Some("unmerged_R2.fastq.gz".to_string()),
            reads_r1: 100,
            reads_r2: 100,
            reads_merged: 92,
            reads_unmerged: 8,
            merge_rate: 0.92,
            runtime_s: Some(2.1),
            memory_mb: Some(48.0),
            raw_backend_report: Some("pear.log".to_string()),
            raw_backend_report_format: Some("pear_log".to_string()),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: MergePairsReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "pear");
        assert_eq!(decoded.merge_overlap, Some(20));
        assert_eq!(decoded.reads_merged, 92);
    }

    #[test]
    fn merge_pairs_report_derives_canonical_pair_counts() {
        let report = MergePairsReportV1 {
            schema_version: MERGE_PAIRS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.merge_pairs".to_string(),
            stage_id: "fastq.merge_pairs".to_string(),
            tool_id: "pear".to_string(),
            paired_mode: PairedMode::PairedEnd,
            merge_engine: MergeEngine::Pear,
            threads: 4,
            merge_overlap: Some(20),
            min_len: Some(80),
            unmerged_read_policy: UnmergedReadPolicy::EmitUnmergedPairs,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: "reads_R2.fastq.gz".to_string(),
            merged_reads: "merged.fastq.gz".to_string(),
            unmerged_reads_r1: Some("unmerged_R1.fastq.gz".to_string()),
            unmerged_reads_r2: Some("unmerged_R2.fastq.gz".to_string()),
            reads_r1: 100,
            reads_r2: 98,
            reads_merged: 92,
            reads_unmerged: 5,
            merge_rate: 0.94,
            runtime_s: Some(2.1),
            memory_mb: Some(48.0),
            raw_backend_report: None,
            raw_backend_report_format: None,
        };

        assert_eq!(
            report.canonical_pair_counts(),
            MergePairCountsV1 {
                input_pair_count: 98,
                merged_pair_count: 92,
                unmerged_pair_count: 5,
                discarded_pair_count: 1,
            }
        );
    }
}

#[cfg(test)]
mod local_smoke_contract_tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn local_smoke_observation_preserves_wire_shape_and_producer_identity() -> Result<()> {
        let original: serde_json::Value = serde_json::from_str(
            r#"{
    "schema_version": "bijux.fastq.merge_pairs.local_smoke.report.v1",
    "stage_id": "fastq.merge_pairs",
    "sample_id": "authored_contract_value",
    "planned_tool_id": "planned_backend",
    "report_tool_id": "local_emitter",
    "merge_overlap": 1,
    "min_length": 1,
    "input_pair_count": 1,
    "merged_count": 1,
    "unmerged_r1_count": 1,
    "unmerged_r2_count": 1,
    "discarded_count": 1,
    "merged_fastq_gz": "authored_contract_value",
    "unmerged_r1_fastq_gz": null,
    "unmerged_r2_fastq_gz": null,
    "case_report_json": "authored_contract_value",
    "raw_backend_report": null
}"#,
        )?;
        let report: LocalMergePairsSmokeReportV1 = serde_json::from_value(original.clone())?;
        assert_eq!(report.schema_version, LOCAL_MERGE_PAIRS_SMOKE_REPORT_SCHEMA_VERSION);
        assert_ne!(report.planned_tool_id, report.report_tool_id);
        assert_eq!(serde_json::to_value(&report)?, original);
        let mut unsupported = original;
        unsupported["unrecognized"] = serde_json::json!(true);
        assert!(serde_json::from_value::<LocalMergePairsSmokeReportV1>(unsupported).is_err());
        let schema = serde_json::to_value(schemars::schema_for!(LocalMergePairsSmokeReportV1))?;
        assert_eq!(schema["additionalProperties"], serde_json::json!(false));
        Ok(())
    }
}
