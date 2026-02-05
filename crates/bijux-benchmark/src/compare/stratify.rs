//! Owner: bijux-bench
//! Stratification helpers for comparisons.

#[derive(Debug, Clone, serde::Serialize)]
pub struct CompareStratum {
    pub dataset_id: String,
    pub dataset_class: String,
    pub read_layout: String,
    pub stage_id: String,
    pub tool_id: String,
    pub params_hash: String,
}
