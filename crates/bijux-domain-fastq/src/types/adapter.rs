//! Owner: bijux-domain-fastq
//! Adapter trimming report contracts.

use crate::types::ToolReferenceV1;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdapterTrimmingReportV1 {
    pub schema_version: String,
    pub reads_with_adapter: u64,
    pub total_reads: u64,
    pub bases_trimmed_total: u64,
    pub per_adapter_counts: std::collections::BTreeMap<String, u64>,
    pub top_k_adapters: Vec<AdapterContributionV1>,
    pub tool: ToolReferenceV1,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdapterContributionV1 {
    pub id: String,
    pub count: u64,
}
