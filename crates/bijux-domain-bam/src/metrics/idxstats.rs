use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IdxstatsContigV1 {
    pub contig: String,
    pub length: u64,
    pub mapped: u64,
    pub unmapped: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct IdxstatsSummaryV1 {
    pub contigs: Vec<IdxstatsContigV1>,
    pub total_mapped: u64,
    pub total_unmapped: u64,
    pub reference_mismatch: bool,
}

impl IdxstatsSummaryV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            contigs: Vec::new(),
            total_mapped: 0,
            total_unmapped: 0,
            reference_mismatch: false,
        }
    }
}
