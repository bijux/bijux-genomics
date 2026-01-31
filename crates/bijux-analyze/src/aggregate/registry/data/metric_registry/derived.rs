//! Owner: bijux-analyze
//! Derived metric registry.

use super::super::super::ids::{DerivedMetricId, DerivedMetricSpec, MetricDirection, MetricRange};

pub const DERIVED_METRIC_REGISTRY: [DerivedMetricSpec; 4] = [
    DerivedMetricSpec {
        id: DerivedMetricId::ReadRetention,
        name: "read_retention",
        meaning: "reads_out / reads_in",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange { min: 0.0, max: 1.0 }),
        stages: &["fastq.trim", "fastq.filter", "fastq.correct", "fastq.umi"],
    },
    DerivedMetricSpec {
        id: DerivedMetricId::BaseRetention,
        name: "base_retention",
        meaning: "bases_out / bases_in",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange { min: 0.0, max: 1.0 }),
        stages: &["fastq.trim", "fastq.filter", "fastq.correct"],
    },
    DerivedMetricSpec {
        id: DerivedMetricId::MergeEfficiency,
        name: "merge_efficiency",
        meaning: "reads_merged / min(reads_r1, reads_r2)",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange { min: 0.0, max: 1.0 }),
        stages: &["fastq.merge"],
    },
    DerivedMetricSpec {
        id: DerivedMetricId::ErrorReductionProxy,
        name: "error_reduction_proxy",
        meaning: "max(0, mean_q_after - mean_q_before)",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: 45.0,
        }),
        stages: &["fastq.trim", "fastq.filter", "fastq.correct"],
    },
];
