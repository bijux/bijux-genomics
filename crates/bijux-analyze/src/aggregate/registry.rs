//! Owner: bijux-analyze
//! Metric registry and schema definitions.
//! Owns metric specs and registry lookup functions.
//! Must not perform IO or depend on pipeline/report layers.
//! Invariants: registry constants are exhaustive for known stages.

use crate::aggregate::registry_defs as defs;

pub use defs::*;

#[must_use]
pub fn stage_metric_spec(kind: defs::StageMetricKind) -> defs::StageMetricSpec {
    match kind {
        defs::StageMetricKind::FastqTrim => defs::StageMetricSpec {
            stage: "fastq.trim",
            version: 2,
            metrics: &defs::FASTQ_TRIM_METRICS,
            invariants: &defs::FASTQ_TRIM_INVARIANTS,
        },
        defs::StageMetricKind::FastqValidate => defs::StageMetricSpec {
            stage: "fastq.validate_pre",
            version: 1,
            metrics: &defs::FASTQ_VALIDATE_METRICS,
            invariants: &defs::FASTQ_VALIDATE_INVARIANTS,
        },
        defs::StageMetricKind::FastqFilter => defs::StageMetricSpec {
            stage: "fastq.filter",
            version: 2,
            metrics: &defs::FASTQ_FILTER_METRICS,
            invariants: &defs::FASTQ_FILTER_INVARIANTS,
        },
        defs::StageMetricKind::FastqMerge => defs::StageMetricSpec {
            stage: "fastq.merge",
            version: 1,
            metrics: &defs::FASTQ_MERGE_METRICS,
            invariants: &defs::FASTQ_MERGE_INVARIANTS,
        },
        defs::StageMetricKind::FastqCorrect => defs::StageMetricSpec {
            stage: "fastq.correct",
            version: 1,
            metrics: &defs::FASTQ_CORRECT_METRICS,
            invariants: &defs::FASTQ_CORRECT_INVARIANTS,
        },
        defs::StageMetricKind::FastqQcPost => defs::StageMetricSpec {
            stage: "fastq.qc_post",
            version: 1,
            metrics: &defs::FASTQ_QC_POST_METRICS,
            invariants: &defs::FASTQ_QC_POST_INVARIANTS,
        },
        defs::StageMetricKind::FastqUmi => defs::StageMetricSpec {
            stage: "fastq.umi",
            version: 1,
            metrics: &defs::FASTQ_UMI_METRICS,
            invariants: &defs::FASTQ_UMI_INVARIANTS,
        },
        defs::StageMetricKind::FastqScreen => defs::StageMetricSpec {
            stage: "fastq.screen",
            version: 1,
            metrics: &defs::FASTQ_SCREEN_METRICS,
            invariants: &defs::FASTQ_SCREEN_INVARIANTS,
        },
        defs::StageMetricKind::FastqStats => defs::StageMetricSpec {
            stage: "fastq.stats_neutral",
            version: 1,
            metrics: &defs::FASTQ_STATS_METRICS,
            invariants: &defs::FASTQ_STATS_INVARIANTS,
        },
    }
}

pub struct StageMetricRegistry;

impl StageMetricRegistry {
    #[must_use]
    pub fn kind_for_stage(stage_id: &str) -> Option<defs::StageMetricKind> {
        defs::metric_kind_for_stage(stage_id)
    }

    #[must_use]
    pub fn spec_for_stage(stage_id: &str) -> Option<defs::StageMetricSpec> {
        Self::kind_for_stage(stage_id).map(stage_metric_spec)
    }
}

/// Lookup a metric spec by id.
///
/// # Panics
/// Panics if the metric id is not present in the registry.
#[must_use]
pub fn metric_spec(metric_id: defs::MetricId) -> defs::MetricSpec {
    defs::METRIC_REGISTRY_CORE
        .iter()
        .chain(defs::METRIC_REGISTRY_FASTQ.iter())
        .chain(defs::METRIC_REGISTRY_QUALITY.iter())
        .copied()
        .find(|spec| spec.id == metric_id)
        .unwrap_or_else(|| panic!("missing metric spec for {metric_id:?}"))
}

/// Lookup a derived metric spec by id.
///
/// # Panics
/// Panics if the derived metric id is not present in the registry.
#[must_use]
pub fn derived_metric_spec(metric_id: defs::DerivedMetricId) -> defs::DerivedMetricSpec {
    defs::DERIVED_METRIC_REGISTRY
        .iter()
        .copied()
        .find(|spec| spec.id == metric_id)
        .unwrap_or_else(|| panic!("missing derived metric spec for {metric_id:?}"))
}

#[must_use]
pub fn derived_metrics_for_stage(stage_id: &str) -> Vec<DerivedMetricSpec> {
    defs::DERIVED_METRIC_REGISTRY
        .iter()
        .copied()
        .filter(|spec| spec.stages.iter().any(|stage| stage == &stage_id))
        .collect()
}
