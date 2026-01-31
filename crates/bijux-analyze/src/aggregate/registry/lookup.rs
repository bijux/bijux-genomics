//! Owner: bijux-analyze
//! Metric registry lookup helpers.

use super::data::{
    metric_kind_for_stage, DERIVED_METRIC_REGISTRY, FASTQ_CORRECT_INVARIANTS,
    FASTQ_CORRECT_METRICS, FASTQ_FILTER_INVARIANTS, FASTQ_FILTER_METRICS, FASTQ_MERGE_INVARIANTS,
    FASTQ_MERGE_METRICS, FASTQ_QC_POST_INVARIANTS, FASTQ_QC_POST_METRICS, FASTQ_SCREEN_INVARIANTS,
    FASTQ_SCREEN_METRICS, FASTQ_STATS_INVARIANTS, FASTQ_STATS_METRICS, FASTQ_TRIM_INVARIANTS,
    FASTQ_TRIM_METRICS, FASTQ_UMI_INVARIANTS, FASTQ_UMI_METRICS, FASTQ_VALIDATE_INVARIANTS,
    FASTQ_VALIDATE_METRICS, METRIC_REGISTRY_PART1, METRIC_REGISTRY_PART2, METRIC_REGISTRY_PART3,
};
use super::ids::{
    DerivedMetricId, DerivedMetricSpec, MetricId, MetricSpec, StageMetricKind, StageMetricSpec,
};

#[must_use]
pub fn stage_metric_spec(kind: StageMetricKind) -> StageMetricSpec {
    match kind {
        StageMetricKind::FastqTrim => StageMetricSpec {
            stage: "fastq.trim",
            version: 2,
            metrics: &FASTQ_TRIM_METRICS,
            invariants: &FASTQ_TRIM_INVARIANTS,
        },
        StageMetricKind::FastqValidate => StageMetricSpec {
            stage: "fastq.validate_pre",
            version: 1,
            metrics: &FASTQ_VALIDATE_METRICS,
            invariants: &FASTQ_VALIDATE_INVARIANTS,
        },
        StageMetricKind::FastqFilter => StageMetricSpec {
            stage: "fastq.filter",
            version: 2,
            metrics: &FASTQ_FILTER_METRICS,
            invariants: &FASTQ_FILTER_INVARIANTS,
        },
        StageMetricKind::FastqMerge => StageMetricSpec {
            stage: "fastq.merge",
            version: 1,
            metrics: &FASTQ_MERGE_METRICS,
            invariants: &FASTQ_MERGE_INVARIANTS,
        },
        StageMetricKind::FastqCorrect => StageMetricSpec {
            stage: "fastq.correct",
            version: 1,
            metrics: &FASTQ_CORRECT_METRICS,
            invariants: &FASTQ_CORRECT_INVARIANTS,
        },
        StageMetricKind::FastqQcPost => StageMetricSpec {
            stage: "fastq.qc_post",
            version: 1,
            metrics: &FASTQ_QC_POST_METRICS,
            invariants: &FASTQ_QC_POST_INVARIANTS,
        },
        StageMetricKind::FastqUmi => StageMetricSpec {
            stage: "fastq.umi",
            version: 1,
            metrics: &FASTQ_UMI_METRICS,
            invariants: &FASTQ_UMI_INVARIANTS,
        },
        StageMetricKind::FastqScreen => StageMetricSpec {
            stage: "fastq.screen",
            version: 1,
            metrics: &FASTQ_SCREEN_METRICS,
            invariants: &FASTQ_SCREEN_INVARIANTS,
        },
        StageMetricKind::FastqStats => StageMetricSpec {
            stage: "fastq.stats_neutral",
            version: 1,
            metrics: &FASTQ_STATS_METRICS,
            invariants: &FASTQ_STATS_INVARIANTS,
        },
    }
}

pub struct StageMetricRegistry;

impl StageMetricRegistry {
    #[must_use]
    pub fn kind_for_stage(stage_id: &str) -> Option<StageMetricKind> {
        metric_kind_for_stage(stage_id)
    }

    #[must_use]
    pub fn spec_for_stage(stage_id: &str) -> Option<StageMetricSpec> {
        Self::kind_for_stage(stage_id).map(stage_metric_spec)
    }
}

/// Lookup a metric spec by id.
///
/// # Panics
/// Panics if the metric id is not present in the registry.
#[must_use]
pub fn metric_spec(metric_id: MetricId) -> MetricSpec {
    METRIC_REGISTRY_PART1
        .iter()
        .chain(METRIC_REGISTRY_PART2.iter())
        .chain(METRIC_REGISTRY_PART3.iter())
        .copied()
        .find(|spec| spec.id == metric_id)
        .unwrap_or_else(|| panic!("missing metric spec for {metric_id:?}"))
}

/// Lookup a derived metric spec by id.
///
/// # Panics
/// Panics if the derived metric id is not present in the registry.
#[must_use]
pub fn derived_metric_spec(metric_id: DerivedMetricId) -> DerivedMetricSpec {
    DERIVED_METRIC_REGISTRY
        .iter()
        .copied()
        .find(|spec| spec.id == metric_id)
        .unwrap_or_else(|| panic!("missing derived metric spec for {metric_id:?}"))
}

#[must_use]
pub fn derived_metrics_for_stage(stage_id: &str) -> Vec<DerivedMetricSpec> {
    DERIVED_METRIC_REGISTRY
        .iter()
        .copied()
        .filter(|spec| spec.stages.iter().any(|stage| stage == &stage_id))
        .collect()
}
