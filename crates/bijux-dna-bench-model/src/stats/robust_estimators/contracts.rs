//! Owner: bijux-dna-bench-model
//! Robust statistics result contracts.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RobustStats {
    pub n: usize,
    pub median: f64,
    pub mad: f64,
    pub iqr: f64,
    pub trimmed_mean: f64,
}
