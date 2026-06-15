//! Owner: bijux-dna-analyze
//! Metric registry and schema definitions.
//! Owns metric specs and registry lookup functions.
//! Must not perform IO or depend on pipeline/report layers.
//! Invariants: registry constants are exhaustive for known stages.

use super::{defs, fields};

fn stage_metric_spec_entry(
    stage: &'static str,
    version: i32,
    metrics: &'static [defs::MetricId],
    invariants: &'static [&'static str],
) -> defs::StageMetricSpec {
    defs::StageMetricSpec { stage, version, metrics, invariants }
}

fn stage_metric_spec_transform(kind: defs::StageMetricKind) -> Option<defs::StageMetricSpec> {
    match kind {
        defs::StageMetricKind::FastqTrim => Some(stage_metric_spec_entry(
            "fastq.trim_reads",
            2,
            &fields::FASTQ_TRIM_METRICS,
            &fields::FASTQ_TRIM_INVARIANTS,
        )),
        defs::StageMetricKind::FastqTrimPolyg => Some(stage_metric_spec_entry(
            "fastq.trim_polyg_tails",
            1,
            &fields::FASTQ_TRIM_POLYG_METRICS,
            &fields::FASTQ_TRIM_INVARIANTS,
        )),
        defs::StageMetricKind::FastqTrimTerminalDamage => Some(stage_metric_spec_entry(
            "fastq.trim_terminal_damage",
            1,
            &fields::FASTQ_TRIM_TERMINAL_DAMAGE_METRICS,
            &fields::FASTQ_TRIM_INVARIANTS,
        )),
        defs::StageMetricKind::FastqFilter => Some(stage_metric_spec_entry(
            "fastq.filter_reads",
            2,
            &fields::FASTQ_FILTER_METRICS,
            &fields::FASTQ_FILTER_INVARIANTS,
        )),
        defs::StageMetricKind::FastqLowComplexity => Some(stage_metric_spec_entry(
            "fastq.filter_low_complexity",
            1,
            &fields::FASTQ_LOW_COMPLEXITY_METRICS,
            &fields::FASTQ_LOW_COMPLEXITY_INVARIANTS,
        )),
        defs::StageMetricKind::FastqDeduplicate => Some(stage_metric_spec_entry(
            "fastq.remove_duplicates",
            1,
            &fields::FASTQ_DEDUPLICATE_METRICS,
            &fields::FASTQ_DEDUPLICATE_INVARIANTS,
        )),
        defs::StageMetricKind::FastqMerge => Some(stage_metric_spec_entry(
            "fastq.merge_pairs",
            1,
            &fields::FASTQ_MERGE_METRICS,
            &fields::FASTQ_MERGE_INVARIANTS,
        )),
        defs::StageMetricKind::FastqCorrect => Some(stage_metric_spec_entry(
            "fastq.correct_errors",
            1,
            &fields::FASTQ_CORRECT_METRICS,
            &fields::FASTQ_CORRECT_INVARIANTS,
        )),
        defs::StageMetricKind::FastqQcPost => Some(stage_metric_spec_entry(
            "fastq.report_qc",
            1,
            &fields::FASTQ_QC_POST_METRICS,
            &fields::FASTQ_QC_POST_INVARIANTS,
        )),
        _ => None,
    }
}

fn stage_metric_spec_reference(kind: defs::StageMetricKind) -> Option<defs::StageMetricSpec> {
    match kind {
        defs::StageMetricKind::FastqIndexReference => Some(stage_metric_spec_entry(
            "fastq.index_reference",
            1,
            &fields::FASTQ_INDEX_REFERENCE_METRICS,
            &fields::FASTQ_INDEX_REFERENCE_INVARIANTS,
        )),
        defs::StageMetricKind::FastqDepleteHost => Some(stage_metric_spec_entry(
            "fastq.deplete_host",
            1,
            &fields::FASTQ_DEPLETE_HOST_METRICS,
            &fields::FASTQ_DEPLETE_HOST_INVARIANTS,
        )),
        defs::StageMetricKind::FastqDepleteReferenceContaminants => Some(stage_metric_spec_entry(
            "fastq.deplete_reference_contaminants",
            1,
            &fields::FASTQ_DEPLETE_REFERENCE_CONTAMINANTS_METRICS,
            &fields::FASTQ_DEPLETE_REFERENCE_CONTAMINANTS_INVARIANTS,
        )),
        defs::StageMetricKind::FastqDepleteRrna => Some(stage_metric_spec_entry(
            "fastq.deplete_rrna",
            1,
            &fields::FASTQ_DEPLETE_RRNA_METRICS,
            &fields::FASTQ_DEPLETE_RRNA_INVARIANTS,
        )),
        defs::StageMetricKind::FastqScreen => Some(stage_metric_spec_entry(
            "fastq.screen_taxonomy",
            1,
            &fields::FASTQ_SCREEN_METRICS,
            &fields::FASTQ_SCREEN_INVARIANTS,
        )),
        defs::StageMetricKind::FastqNormalizePrimers => Some(stage_metric_spec_entry(
            "fastq.normalize_primers",
            1,
            &fields::FASTQ_NORMALIZE_PRIMERS_METRICS,
            &fields::FASTQ_NORMALIZE_PRIMERS_INVARIANTS,
        )),
        _ => None,
    }
}

fn stage_metric_spec_profile(kind: defs::StageMetricKind) -> Option<defs::StageMetricSpec> {
    match kind {
        defs::StageMetricKind::FastqValidate => Some(stage_metric_spec_entry(
            "fastq.validate_reads",
            1,
            &fields::FASTQ_VALIDATE_METRICS,
            &fields::FASTQ_VALIDATE_INVARIANTS,
        )),
        defs::StageMetricKind::FastqDetectAdapters => Some(stage_metric_spec_entry(
            "fastq.detect_adapters",
            2,
            &fields::FASTQ_DETECT_ADAPTERS_METRICS,
            &fields::FASTQ_DETECT_ADAPTERS_INVARIANTS,
        )),
        defs::StageMetricKind::FastqUmi => Some(stage_metric_spec_entry(
            "fastq.extract_umis",
            1,
            &fields::FASTQ_UMI_METRICS,
            &fields::FASTQ_UMI_INVARIANTS,
        )),
        defs::StageMetricKind::FastqStats => Some(stage_metric_spec_entry(
            "fastq.profile_reads",
            1,
            &fields::FASTQ_STATS_METRICS,
            &fields::FASTQ_STATS_INVARIANTS,
        )),
        defs::StageMetricKind::FastqReadLengths => Some(stage_metric_spec_entry(
            "fastq.profile_read_lengths",
            1,
            &fields::FASTQ_READ_LENGTH_METRICS,
            &fields::FASTQ_READ_LENGTH_INVARIANTS,
        )),
        defs::StageMetricKind::FastqOverrepresented => Some(stage_metric_spec_entry(
            "fastq.profile_overrepresented_sequences",
            1,
            &fields::FASTQ_OVERREPRESENTED_METRICS,
            &fields::FASTQ_OVERREPRESENTED_INVARIANTS,
        )),
        _ => None,
    }
}

#[must_use]
/// # Panics
/// Panics if a known stage kind is missing from the metric registry mapping.
pub fn stage_metric_spec(kind: defs::StageMetricKind) -> defs::StageMetricSpec {
    stage_metric_spec_transform(kind)
        .or_else(|| stage_metric_spec_reference(kind))
        .or_else(|| stage_metric_spec_profile(kind))
        .unwrap_or_else(|| panic!("missing stage metric spec for {kind:?}"))
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
        .chain(fields::METRIC_REGISTRY_QUALITY.iter())
        .chain(fields::METRIC_REGISTRY_SCREENING_AND_REFERENCE.iter())
        .chain(fields::METRIC_REGISTRY_PROFILING_AND_REPORTING.iter())
        .chain(fields::METRIC_REGISTRY_PROCESSING_AND_VALIDATION.iter())
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
