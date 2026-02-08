//! Owner: bijux-dna-benchmark
//! Benchmark summary model (versioned).
//! Owns aggregate stats and completeness for observations.
//! Must not perform IO or depend on compare/gate logic.

use serde::{Deserialize, Serialize};

use crate::stats::robust_estimators::RobustStats;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricSummary {
    pub metric_id: String,
    pub n: usize,
    pub stats: RobustStats,
    pub ci_low: Option<f64>,
    pub ci_high: Option<f64>,
    pub outlier_count: usize,
    pub outlier_replicates: Vec<String>,
    pub practical_threshold: Option<f64>,
    pub power_warning: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SummaryRow {
    pub dataset_id: String,
    pub dataset_class: String,
    pub read_layout: String,
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
    pub strata: Vec<SummaryStratum>,
    pub warnings: Vec<String>,
    pub scientifically_invalid: bool,
    pub invalid_reasons: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SummaryStratum {
    pub stage_id: String,
    pub dataset_class: String,
    pub row_count: usize,
    pub low_power_count: usize,
}

impl BenchmarkSummary {
    #[must_use]
    pub fn v1(
        suite_id: String,
        rows: Vec<SummaryRow>,
        strata: Vec<SummaryStratum>,
        warnings: Vec<String>,
    ) -> Self {
        Self {
            schema_version: "bijux.bench.summary.v1".to_string(),
            suite_id,
            rows,
            strata,
            warnings,
            scientifically_invalid: false,
            invalid_reasons: Vec::new(),
        }
    }
}
