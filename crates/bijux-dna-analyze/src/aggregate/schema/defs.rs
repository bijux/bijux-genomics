// Owner: bijux-dna-analyze
// Metric registry definitions and stage metric sets.

use serde::{Deserialize, Serialize};

pub use bijux_dna_core::metrics::{DerivedMetricId, MetricId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageMetricKind {
    FastqTrim,
    FastqValidate,
    FastqFilter,
    FastqMerge,
    FastqCorrect,
    FastqQcPost,
    FastqUmi,
    FastqScreen,
    FastqStats,
}

#[derive(Debug, Clone, Copy)]
pub enum MetricDirection {
    HigherBetter,
    LowerBetter,
    Neutral,
}

#[derive(Debug, Clone, Copy)]
pub struct MetricRange {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct MetricSpec {
    pub id: MetricId,
    pub name: &'static str,
    pub meaning: &'static str,
    pub direction: MetricDirection,
    pub range: Option<MetricRange>,
    pub stages: &'static [&'static str],
    pub measured: bool,
    pub derived: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct DerivedMetricSpec {
    pub id: DerivedMetricId,
    pub name: &'static str,
    pub meaning: &'static str,
    pub direction: MetricDirection,
    pub range: Option<MetricRange>,
    pub stages: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub struct StageMetricSpec {
    pub stage: &'static str,
    pub version: i32,
    pub metrics: &'static [MetricId],
    pub invariants: &'static [&'static str],
}
