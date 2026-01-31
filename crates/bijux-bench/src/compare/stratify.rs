//! Owner: bijux-bench
//! Stratification helpers for comparisons.

#[derive(Debug, Clone, serde::Serialize)]
pub struct CompareStratum {
    pub dataset_id: String,
    pub platform: String,
    pub preset: Option<String>,
}
