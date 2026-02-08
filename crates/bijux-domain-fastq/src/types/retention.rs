//! Owner: bijux-domain-fastq
//! Retention report contracts.

use crate::types::ToolReferenceV1;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetentionReportV1 {
    pub schema_version: String,
    pub definition: String,
    pub numerator: serde_json::Value,
    pub denominator: serde_json::Value,
    pub units: String,
    pub scope: String,
    pub stage_boundary: String,
    pub tool: ToolReferenceV1,
    pub raw_reads_total: Option<u64>,
}
