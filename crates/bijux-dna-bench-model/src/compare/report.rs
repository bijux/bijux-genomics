//! Owner: bijux-dna-bench-model
//! Comparison report contracts.

use crate::compare::stratify::CompareStratum;

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricDiff {
    pub metric_id: String,
    pub absolute: f64,
    pub relative: Option<f64>,
    pub practical: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CompareReport {
    pub suite_a: String,
    pub suite_b: String,
    pub diffs: Vec<MetricDiff>,
    pub strata: Vec<CompareStratum>,
}
