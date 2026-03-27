//! Owner: bijux-dna-analyze
//! Metric registry and schema definitions.
//! Owns metric specs and registry lookup functions.
//! Must not perform IO or depend on pipeline/report layers.
//! Invariants: registry constants are exhaustive for known stages.

use super::{defs, fields};

#[must_use]
pub fn stage_metric_spec(kind: defs::StageMetricKind) -> defs::StageMetricSpec {
    match kind {
        defs::StageMetricKind::FastqTrim => defs::StageMetricSpec {
            stage: "fastq.trim_reads",
            version: 2,
            metrics: &fields::FASTQ_TRIM_METRICS,
            invariants: &fields::FASTQ_TRIM_INVARIANTS,
        },
        defs::StageMetricKind::FastqTrimPolyg => defs::StageMetricSpec {
            stage: "fastq.trim_polyg_tails",
            version: 1,
            metrics: &fields::FASTQ_TRIM_POLYG_METRICS,
            invariants: &fields::FASTQ_TRIM_INVARIANTS,
        },
        defs::StageMetricKind::FastqTrimTerminalDamage => defs::StageMetricSpec {
            stage: "fastq.trim_terminal_damage",
            version: 1,
            metrics: &fields::FASTQ_TRIM_TERMINAL_DAMAGE_METRICS,
            invariants: &fields::FASTQ_TRIM_INVARIANTS,
        },
        defs::StageMetricKind::FastqValidate => defs::StageMetricSpec {
            stage: "fastq.validate_reads",
            version: 1,
            metrics: &fields::FASTQ_VALIDATE_METRICS,
            invariants: &fields::FASTQ_VALIDATE_INVARIANTS,
        },
        defs::StageMetricKind::FastqDetectAdapters => defs::StageMetricSpec {
            stage: "fastq.detect_adapters",
            version: 1,
            metrics: &fields::FASTQ_DETECT_ADAPTERS_METRICS,
            invariants: &fields::FASTQ_DETECT_ADAPTERS_INVARIANTS,
        },
        defs::StageMetricKind::FastqFilter => defs::StageMetricSpec {
            stage: "fastq.filter_reads",
            version: 2,
            metrics: &fields::FASTQ_FILTER_METRICS,
            invariants: &fields::FASTQ_FILTER_INVARIANTS,
        },
        defs::StageMetricKind::FastqLowComplexity => defs::StageMetricSpec {
            stage: "fastq.filter_low_complexity",
            version: 1,
            metrics: &fields::FASTQ_LOW_COMPLEXITY_METRICS,
            invariants: &fields::FASTQ_LOW_COMPLEXITY_INVARIANTS,
        },
        defs::StageMetricKind::FastqMerge => defs::StageMetricSpec {
            stage: "fastq.merge_pairs",
            version: 1,
            metrics: &fields::FASTQ_MERGE_METRICS,
            invariants: &fields::FASTQ_MERGE_INVARIANTS,
        },
        defs::StageMetricKind::FastqCorrect => defs::StageMetricSpec {
            stage: "fastq.correct_errors",
            version: 1,
            metrics: &fields::FASTQ_CORRECT_METRICS,
            invariants: &fields::FASTQ_CORRECT_INVARIANTS,
        },
        defs::StageMetricKind::FastqQcPost => defs::StageMetricSpec {
            stage: "fastq.report_qc",
            version: 1,
            metrics: &fields::FASTQ_QC_POST_METRICS,
            invariants: &fields::FASTQ_QC_POST_INVARIANTS,
        },
        defs::StageMetricKind::FastqUmi => defs::StageMetricSpec {
            stage: "fastq.extract_umis",
            version: 1,
            metrics: &fields::FASTQ_UMI_METRICS,
            invariants: &fields::FASTQ_UMI_INVARIANTS,
        },
        defs::StageMetricKind::FastqScreen => defs::StageMetricSpec {
            stage: "fastq.screen_taxonomy",
            version: 1,
            metrics: &fields::FASTQ_SCREEN_METRICS,
            invariants: &fields::FASTQ_SCREEN_INVARIANTS,
        },
        defs::StageMetricKind::FastqStats => defs::StageMetricSpec {
            stage: "fastq.profile_reads",
            version: 1,
            metrics: &fields::FASTQ_STATS_METRICS,
            invariants: &fields::FASTQ_STATS_INVARIANTS,
        },
        defs::StageMetricKind::FastqReadLengths => defs::StageMetricSpec {
            stage: "fastq.profile_read_lengths",
            version: 1,
            metrics: &fields::FASTQ_READ_LENGTH_METRICS,
            invariants: &fields::FASTQ_READ_LENGTH_INVARIANTS,
        },
        defs::StageMetricKind::FastqOverrepresented => defs::StageMetricSpec {
            stage: "fastq.profile_overrepresented_sequences",
            version: 1,
            metrics: &fields::FASTQ_OVERREPRESENTED_METRICS,
            invariants: &fields::FASTQ_OVERREPRESENTED_INVARIANTS,
        },
    }
}

pub struct StageMetricRegistry;

impl StageMetricRegistry {
    #[must_use]
    pub fn kind_for_stage(stage_id: &str) -> Option<defs::StageMetricKind> {
        fields::metric_kind_for_stage(stage_id)
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
    fields::METRIC_REGISTRY_CORE
        .iter()
        .chain(fields::METRIC_REGISTRY_FASTQ.iter())
        .chain(fields::METRIC_REGISTRY_QUALITY.iter())
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
    fields::DERIVED_METRIC_REGISTRY
        .iter()
        .copied()
        .find(|spec| spec.id == metric_id)
        .unwrap_or_else(|| panic!("missing derived metric spec for {metric_id:?}"))
}

#[must_use]
pub fn derived_metrics_for_stage(stage_id: &str) -> Vec<defs::DerivedMetricSpec> {
    fields::DERIVED_METRIC_REGISTRY
        .iter()
        .copied()
        .filter(|spec| spec.stages.iter().any(|stage| stage == &stage_id))
        .collect()
}
