//! Owner: bijux-bench
//! Benchmark summary model (versioned).
//! Owns aggregate stats and completeness for observations.
//! Must not perform IO or depend on compare/gate logic.

use serde::{Deserialize, Serialize};

use crate::stats::robust::RobustStats;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricSummary {
    pub metric_id: String,
    pub stats: RobustStats,
    pub ci_low: Option<f64>,
    pub ci_high: Option<f64>,
    pub outlier_count: usize,
    pub outlier_replicates: Vec<String>,
    pub practical_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SummaryRow {
    pub dataset_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub params_hash: String,
    pub runtime: MetricSummary,
    pub memory: MetricSummary,
    pub metrics: Vec<MetricSummary>,
    pub failure_rate: f64,
    pub completeness: f64,
    pub n_effective: usize,
    pub low_power: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkSummary {
    pub schema_version: String,
    pub suite_id: String,
    pub rows: Vec<SummaryRow>,
    pub warnings: Vec<String>,
}

impl BenchmarkSummary {
    #[must_use]
    pub fn v1(suite_id: String, rows: Vec<SummaryRow>, warnings: Vec<String>) -> Self {
        Self {
            schema_version: "bijux.bench.summary.v1".to_string(),
            suite_id,
            rows,
            warnings,
        }
    }
}
