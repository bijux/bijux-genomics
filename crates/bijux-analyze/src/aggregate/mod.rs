use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use bijux_core::measure::ExecutionMetrics;
use bijux_core::metrics::MetricSet;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use thiserror::Error;
use tracing::warn;

pub type Result<T> = std::result::Result<T, BenchError>;

pub trait StageMetricSchema {
    const STAGE: &'static str;
    const VERSION: i32;
    /// Validate the schema invariants.
    ///
    /// # Errors
    /// Returns an error if the schema invariants are violated.
    fn validate(&self) -> Result<()>;
}

fn metric_schema_name(stage: &str, version: i32) -> String {
    format!("{}_v{}", stage.replace('.', "_"), version)
}

fn validate_metric_schema<T>(metrics: &T) -> Result<()>
where
    T: StageMetricSchema + Serialize,
{
    let stage = T::STAGE;
    let spec = StageMetricRegistry::spec_for_stage(stage).ok_or_else(|| {
        BenchError::Validation(format!("missing metric schema for stage {stage}"))
    })?;
    let value = serde_json::to_value(metrics)?;
    let obj = value
        .as_object()
        .ok_or_else(|| BenchError::Validation("metrics must serialize to object".to_string()))?;
    let observed: std::collections::BTreeSet<String> = obj
        .keys()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let expected: std::collections::BTreeSet<String> = spec
        .metrics
        .iter()
        .map(|metric_id| metric_spec(*metric_id).name.to_string())
        .collect();
    if observed != expected {
        return Err(BenchError::Validation(format!(
            "metric schema mismatch for {stage}: observed={observed:?} expected={expected:?}",
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricId {
    RuntimeS,
    MemoryMb,
    ExitCode,
    ReadsIn,
    ReadsOut,
    ReadsDropped,
    ReadsRemovedByN,
    ReadsRemovedByEntropy,
    ReadsRemovedByKmer,
    ReadsTotal,
    ReadsValid,
    ReadsInvalid,
    BasesIn,
    BasesOut,
    BasesTotal,
    PairsIn,
    PairsOut,
    ReadsR1,
    ReadsR2,
    ReadsMerged,
    ReadsUnmerged,
    MeanQBefore,
    MeanQAfter,
    MeanQ,
    MergeRate,
    DedupRate,
    KmerFixRate,
    ContaminationRate,
    GcPercent,
    LengthHistogram,
    DeltaMetrics,
    AdapterPreset,
    AdapterBankId,
    AdapterBankHash,
    AdapterOverrides,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DerivedMetricId {
    ReadRetention,
    BaseRetention,
    MergeEfficiency,
    ErrorReductionProxy,
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
    pub metrics: &'static [MetricId],
    pub invariants: &'static [&'static str],
}

pub const METRIC_REGISTRY: [MetricSpec; 35] = [
    MetricSpec {
        id: MetricId::RuntimeS,
        name: "runtime_s",
        meaning: "Wall-clock runtime in seconds",
        direction: MetricDirection::LowerBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &[
            "fastq.trim",
            "fastq.validate_pre",
            "fastq.filter",
            "fastq.merge",
            "fastq.correct",
            "fastq.qc_post",
            "fastq.umi",
            "fastq.screen",
        ],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::MemoryMb,
        name: "memory_mb",
        meaning: "Peak container memory usage in MB",
        direction: MetricDirection::LowerBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &[
            "fastq.trim",
            "fastq.validate_pre",
            "fastq.filter",
            "fastq.merge",
            "fastq.correct",
            "fastq.qc_post",
            "fastq.umi",
            "fastq.screen",
        ],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ExitCode,
        name: "exit_code",
        meaning: "Process exit code (0 = success)",
        direction: MetricDirection::LowerBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: 255.0,
        }),
        stages: &[
            "fastq.trim",
            "fastq.validate_pre",
            "fastq.filter",
            "fastq.merge",
            "fastq.correct",
            "fastq.qc_post",
            "fastq.umi",
            "fastq.screen",
        ],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ReadsIn,
        name: "reads_in",
        meaning: "Number of input reads",
        direction: MetricDirection::Neutral,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &[
            "fastq.trim",
            "fastq.validate_pre",
            "fastq.filter",
            "fastq.merge",
            "fastq.correct",
            "fastq.umi",
            "fastq.qc_post",
            "fastq.screen",
        ],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ReadsOut,
        name: "reads_out",
        meaning: "Number of output reads",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &[
            "fastq.trim",
            "fastq.validate_pre",
            "fastq.filter",
            "fastq.merge",
            "fastq.correct",
            "fastq.umi",
        ],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ReadsDropped,
        name: "reads_dropped",
        meaning: "Number of reads removed by filtering",
        direction: MetricDirection::LowerBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &["fastq.filter"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ReadsRemovedByN,
        name: "reads_removed_by_n",
        meaning: "Reads removed due to N content",
        direction: MetricDirection::LowerBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &["fastq.filter"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ReadsRemovedByEntropy,
        name: "reads_removed_by_entropy",
        meaning: "Reads removed due to low complexity/entropy",
        direction: MetricDirection::LowerBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &["fastq.filter"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ReadsRemovedByKmer,
        name: "reads_removed_by_kmer",
        meaning: "Reads removed due to k-mer contaminant filtering",
        direction: MetricDirection::LowerBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &["fastq.filter"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ReadsTotal,
        name: "reads_total",
        meaning: "Total number of reads observed",
        direction: MetricDirection::Neutral,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &["fastq.validate_pre"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ReadsValid,
        name: "reads_valid",
        meaning: "Number of reads that passed validation",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &["fastq.validate_pre"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ReadsInvalid,
        name: "reads_invalid",
        meaning: "Number of reads that failed validation",
        direction: MetricDirection::LowerBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &["fastq.validate_pre"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::BasesIn,
        name: "bases_in",
        meaning: "Number of input bases",
        direction: MetricDirection::Neutral,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &[
            "fastq.trim",
            "fastq.validate_pre",
            "fastq.filter",
            "fastq.merge",
            "fastq.correct",
            "fastq.qc_post",
            "fastq.umi",
        ],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::BasesOut,
        name: "bases_out",
        meaning: "Number of output bases",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &[
            "fastq.trim",
            "fastq.validate_pre",
            "fastq.filter",
            "fastq.merge",
            "fastq.correct",
            "fastq.umi",
        ],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::PairsIn,
        name: "pairs_in",
        meaning: "Number of input read pairs",
        direction: MetricDirection::Neutral,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &[
            "fastq.trim",
            "fastq.validate_pre",
            "fastq.filter",
            "fastq.merge",
            "fastq.correct",
            "fastq.umi",
        ],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::PairsOut,
        name: "pairs_out",
        meaning: "Number of output read pairs",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &[
            "fastq.trim",
            "fastq.validate_pre",
            "fastq.filter",
            "fastq.merge",
            "fastq.correct",
            "fastq.umi",
        ],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::BasesTotal,
        name: "bases_total",
        meaning: "Number of bases observed",
        direction: MetricDirection::Neutral,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &["fastq.stats_neutral"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ReadsR1,
        name: "reads_r1",
        meaning: "Number of reads in read 1 input",
        direction: MetricDirection::Neutral,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &["fastq.merge"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ReadsR2,
        name: "reads_r2",
        meaning: "Number of reads in read 2 input",
        direction: MetricDirection::Neutral,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &["fastq.merge"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ReadsMerged,
        name: "reads_merged",
        meaning: "Number of merged reads",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &["fastq.merge"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ReadsUnmerged,
        name: "reads_unmerged",
        meaning: "Number of unmerged reads (per end)",
        direction: MetricDirection::Neutral,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &["fastq.merge"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::MeanQBefore,
        name: "mean_q_before",
        meaning: "Mean Phred quality score before processing",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: 45.0,
        }),
        stages: &["fastq.trim", "fastq.filter", "fastq.correct"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::MeanQAfter,
        name: "mean_q_after",
        meaning: "Mean Phred quality score after processing",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: 45.0,
        }),
        stages: &["fastq.trim", "fastq.filter", "fastq.correct"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::MeanQ,
        name: "mean_q",
        meaning: "Mean Phred quality score across all bases",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: 45.0,
        }),
        stages: &["fastq.validate_pre", "fastq.qc_post"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::MergeRate,
        name: "merge_rate",
        meaning: "Merged reads divided by min(reads_r1, reads_r2)",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange { min: 0.0, max: 1.0 }),
        stages: &["fastq.merge"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::DedupRate,
        name: "dedup_rate",
        meaning: "Fraction of reads removed by UMI deduplication",
        direction: MetricDirection::LowerBetter,
        range: Some(MetricRange { min: 0.0, max: 1.0 }),
        stages: &["fastq.umi"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::KmerFixRate,
        name: "kmer_fix_rate",
        meaning: "Proxy fraction of corrected k-mers",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange { min: 0.0, max: 1.0 }),
        stages: &["fastq.correct"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ContaminationRate,
        name: "contamination_rate",
        meaning: "Estimated contamination rate from screening",
        direction: MetricDirection::LowerBetter,
        range: Some(MetricRange { min: 0.0, max: 1.0 }),
        stages: &["fastq.screen", "fastq.qc_post", "fastq.validate_pre"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::GcPercent,
        name: "gc_percent",
        meaning: "GC percentage across reads",
        direction: MetricDirection::Neutral,
        range: Some(MetricRange {
            min: 0.0,
            max: 100.0,
        }),
        stages: &["fastq.stats_neutral"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::LengthHistogram,
        name: "length_histogram",
        meaning: "Histogram of read lengths",
        direction: MetricDirection::Neutral,
        range: None,
        stages: &["fastq.stats_neutral"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::DeltaMetrics,
        name: "delta_metrics",
        meaning: "Derived delta metrics bundle",
        direction: MetricDirection::Neutral,
        range: None,
        stages: &["fastq.trim", "fastq.filter"],
        measured: false,
        derived: true,
    },
    MetricSpec {
        id: MetricId::AdapterPreset,
        name: "adapter_preset",
        meaning: "Adapter preset name used for trimming",
        direction: MetricDirection::Neutral,
        range: None,
        stages: &["fastq.trim"],
        measured: false,
        derived: false,
    },
    MetricSpec {
        id: MetricId::AdapterBankId,
        name: "adapter_bank_id",
        meaning: "Adapter bank id used for trimming",
        direction: MetricDirection::Neutral,
        range: None,
        stages: &["fastq.trim"],
        measured: false,
        derived: false,
    },
    MetricSpec {
        id: MetricId::AdapterBankHash,
        name: "adapter_bank_hash",
        meaning: "Checksum of adapter bank used for trimming",
        direction: MetricDirection::Neutral,
        range: None,
        stages: &["fastq.trim"],
        measured: false,
        derived: false,
    },
    MetricSpec {
        id: MetricId::AdapterOverrides,
        name: "adapter_overrides",
        meaning: "Adapter enable/disable overrides used for trimming",
        direction: MetricDirection::Neutral,
        range: None,
        stages: &["fastq.trim"],
        measured: false,
        derived: false,
    },
];

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

pub const FASTQ_TRIM_METRICS: [MetricId; 13] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::PairsIn,
    MetricId::PairsOut,
    MetricId::MeanQBefore,
    MetricId::MeanQAfter,
    MetricId::DeltaMetrics,
    MetricId::AdapterPreset,
    MetricId::AdapterBankId,
    MetricId::AdapterBankHash,
    MetricId::AdapterOverrides,
];

pub const FASTQ_VALIDATE_METRICS: [MetricId; 10] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::PairsIn,
    MetricId::PairsOut,
    MetricId::ReadsTotal,
    MetricId::ReadsValid,
    MetricId::ReadsInvalid,
    MetricId::MeanQ,
];

pub const FASTQ_FILTER_METRICS: [MetricId; 13] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::ReadsDropped,
    MetricId::ReadsRemovedByN,
    MetricId::ReadsRemovedByEntropy,
    MetricId::ReadsRemovedByKmer,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::PairsIn,
    MetricId::PairsOut,
    MetricId::MeanQBefore,
    MetricId::MeanQAfter,
    MetricId::DeltaMetrics,
];

pub const FASTQ_MERGE_METRICS: [MetricId; 11] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::PairsIn,
    MetricId::PairsOut,
    MetricId::ReadsR1,
    MetricId::ReadsR2,
    MetricId::ReadsMerged,
    MetricId::ReadsUnmerged,
    MetricId::MergeRate,
];

pub const FASTQ_CORRECT_METRICS: [MetricId; 9] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::PairsIn,
    MetricId::PairsOut,
    MetricId::MeanQBefore,
    MetricId::MeanQAfter,
    MetricId::KmerFixRate,
];

pub const FASTQ_QC_POST_METRICS: [MetricId; 4] = [
    MetricId::ReadsIn,
    MetricId::BasesIn,
    MetricId::MeanQ,
    MetricId::ContaminationRate,
];

pub const FASTQ_UMI_METRICS: [MetricId; 7] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::PairsIn,
    MetricId::PairsOut,
    MetricId::DedupRate,
];

pub const FASTQ_SCREEN_METRICS: [MetricId; 2] = [MetricId::ReadsIn, MetricId::ContaminationRate];

pub const FASTQ_STATS_METRICS: [MetricId; 5] = [
    MetricId::ReadsTotal,
    MetricId::BasesTotal,
    MetricId::MeanQ,
    MetricId::GcPercent,
    MetricId::LengthHistogram,
];

pub const FASTQ_TRIM_INVARIANTS: [&str; 4] = [
    "reads_out <= reads_in",
    "bases_out <= bases_in",
    "mean_q_after >= mean_q_before (warn)",
    "counts are non-negative",
];

pub const FASTQ_VALIDATE_INVARIANTS: [&str; 3] = [
    "reads_valid + reads_invalid == reads_total",
    "mean_q in [0, 45]",
    "counts are non-negative",
];

pub const FASTQ_FILTER_INVARIANTS: [&str; 3] = [
    "reads_out + reads_dropped == reads_in",
    "mean_q_after >= mean_q_before (warn)",
    "counts are non-negative",
];

pub const FASTQ_MERGE_INVARIANTS: [&str; 3] = [
    "reads_merged + reads_unmerged <= min(reads_r1, reads_r2)",
    "merge_rate in [0, 1]",
    "counts are non-negative",
];

pub const FASTQ_CORRECT_INVARIANTS: [&str; 4] = [
    "reads_out == reads_in",
    "bases_out <= bases_in",
    "mean_q_after >= mean_q_before (warn)",
    "counts are non-negative",
];

pub const FASTQ_QC_POST_INVARIANTS: [&str; 3] = [
    "mean_q in [0, 45]",
    "contamination_rate in [0, 1]",
    "counts are non-negative",
];

pub const FASTQ_UMI_INVARIANTS: [&str; 3] = [
    "reads_out <= reads_in",
    "dedup_rate in [0, 1]",
    "counts are non-negative",
];

pub const FASTQ_SCREEN_INVARIANTS: [&str; 2] =
    ["contamination_rate in [0, 1]", "counts are non-negative"];

pub const FASTQ_STATS_INVARIANTS: [&str; 2] = ["mean_q in [0, 45]", "gc_percent in [0, 100]"];

#[must_use]
pub fn metric_kind_for_stage(stage_id: &str) -> Option<StageMetricKind> {
    match stage_id {
        "fastq.trim" => Some(StageMetricKind::FastqTrim),
        "fastq.validate_pre" => Some(StageMetricKind::FastqValidate),
        "fastq.filter" => Some(StageMetricKind::FastqFilter),
        "fastq.merge" => Some(StageMetricKind::FastqMerge),
        "fastq.correct" => Some(StageMetricKind::FastqCorrect),
        "fastq.qc_post" => Some(StageMetricKind::FastqQcPost),
        "fastq.umi" => Some(StageMetricKind::FastqUmi),
        "fastq.screen" => Some(StageMetricKind::FastqScreen),
        "fastq.stats_neutral" => Some(StageMetricKind::FastqStats),
        _ => None,
    }
}

#[must_use]
pub fn stage_metric_spec(kind: StageMetricKind) -> StageMetricSpec {
    match kind {
        StageMetricKind::FastqTrim => StageMetricSpec {
            stage: "fastq.trim",
            metrics: &FASTQ_TRIM_METRICS,
            invariants: &FASTQ_TRIM_INVARIANTS,
        },
        StageMetricKind::FastqValidate => StageMetricSpec {
            stage: "fastq.validate_pre",
            metrics: &FASTQ_VALIDATE_METRICS,
            invariants: &FASTQ_VALIDATE_INVARIANTS,
        },
        StageMetricKind::FastqFilter => StageMetricSpec {
            stage: "fastq.filter",
            metrics: &FASTQ_FILTER_METRICS,
            invariants: &FASTQ_FILTER_INVARIANTS,
        },
        StageMetricKind::FastqMerge => StageMetricSpec {
            stage: "fastq.merge",
            metrics: &FASTQ_MERGE_METRICS,
            invariants: &FASTQ_MERGE_INVARIANTS,
        },
        StageMetricKind::FastqCorrect => StageMetricSpec {
            stage: "fastq.correct",
            metrics: &FASTQ_CORRECT_METRICS,
            invariants: &FASTQ_CORRECT_INVARIANTS,
        },
        StageMetricKind::FastqQcPost => StageMetricSpec {
            stage: "fastq.qc_post",
            metrics: &FASTQ_QC_POST_METRICS,
            invariants: &FASTQ_QC_POST_INVARIANTS,
        },
        StageMetricKind::FastqUmi => StageMetricSpec {
            stage: "fastq.umi",
            metrics: &FASTQ_UMI_METRICS,
            invariants: &FASTQ_UMI_INVARIANTS,
        },
        StageMetricKind::FastqScreen => StageMetricSpec {
            stage: "fastq.screen",
            metrics: &FASTQ_SCREEN_METRICS,
            invariants: &FASTQ_SCREEN_INVARIANTS,
        },
        StageMetricKind::FastqStats => StageMetricSpec {
            stage: "fastq.stats_neutral",
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
    METRIC_REGISTRY
        .iter()
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkContext {
    pub tool: String,
    pub tool_version: String,
    pub image_digest: String,
    pub runner: String,
    pub platform: String,
    pub input_hash: String,
    pub parameters: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqDeltaMetrics {
    pub read_retention: f64,
    pub base_retention: f64,
    pub mean_q_delta: f64,
    pub gc_delta: f64,
}

impl FastqDeltaMetrics {
    /// Validate delta metrics.
    ///
    /// # Errors
    /// Returns an error if delta values are invalid.
    pub fn validate(&self) -> Result<()> {
        if !self.mean_q_delta.is_finite() {
            return Err(BenchError::Validation(
                "mean_q_delta must be finite".to_string(),
            ));
        }
        if !self.gc_delta.is_finite() {
            return Err(BenchError::Validation(
                "gc_delta must be finite".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.read_retention) {
            return Err(BenchError::Validation(
                "read_retention must be within [0, 1]".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.base_retention) {
            return Err(BenchError::Validation(
                "base_retention must be within [0, 1]".to_string(),
            ));
        }
        Ok(())
    }
}

impl Default for FastqDeltaMetrics {
    fn default() -> Self {
        Self {
            read_retention: 0.0,
            base_retention: 0.0,
            mean_q_delta: 0.0,
            gc_delta: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqTrimMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    #[serde(default)]
    pub delta_metrics: FastqDeltaMetrics,
    #[serde(default)]
    pub adapter_preset: Option<String>,
    #[serde(default)]
    pub adapter_bank_id: Option<String>,
    #[serde(default)]
    pub adapter_bank_hash: Option<String>,
    #[serde(default)]
    pub adapter_overrides: Option<serde_json::Value>,
}

impl StageMetricSchema for FastqTrimMetrics {
    const STAGE: &'static str = "fastq.trim";
    const VERSION: i32 = 2;

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation(
                "reads_out must be <= reads_in".to_string(),
            ));
        }
        if self.bases_out > self.bases_in {
            return Err(BenchError::Validation(
                "bases_out must be <= bases_in".to_string(),
            ));
        }
        if self.mean_q_after < self.mean_q_before {
            warn!(
                mean_q_before = self.mean_q_before,
                mean_q_after = self.mean_q_after,
                "mean_q_after is lower than mean_q_before"
            );
        }
        self.delta_metrics.validate()?;
        Ok(())
    }
}

#[must_use]
pub fn metric_set<T>(metrics: T) -> MetricSet<T>
where
    T: StageMetricSchema + Serialize,
{
    MetricSet::new(
        metric_schema_name(T::STAGE, T::VERSION),
        T::VERSION,
        metrics,
    )
}

/// Validate the metric set.
///
/// # Errors
/// Returns an error if the metric schema validation fails.
pub fn validate_metric_set<T>(set: &MetricSet<T>) -> Result<()>
where
    T: StageMetricSchema + Serialize,
{
    let expected_schema = metric_schema_name(T::STAGE, T::VERSION);
    if set.metrics_schema != expected_schema {
        return Err(BenchError::Validation(format!(
            "metric schema mismatch for {}: expected {} got {}",
            T::STAGE,
            expected_schema,
            set.metrics_schema
        )));
    }
    if set.version != T::VERSION {
        return Err(BenchError::Validation(format!(
            "metric version mismatch for {}: expected {} got {}",
            T::STAGE,
            T::VERSION,
            set.version
        )));
    }
    set.metrics.validate()?;
    validate_metric_schema(&set.metrics)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkRecord<T: StageMetricSchema> {
    pub context: BenchmarkContext,
    pub execution: ExecutionMetrics,
    pub metrics: MetricSet<T>,
}

impl<T> BenchmarkRecord<T>
where
    T: StageMetricSchema + Serialize,
{
    /// Validate the record by validating its metrics.
    ///
    /// # Errors
    /// Returns an error if the metric schema validation fails.
    pub fn validate(&self) -> Result<()> {
        self.execution.validate()?;
        validate_metric_set(&self.metrics)
    }
}

#[derive(Debug, Error)]
pub enum BenchError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("measure error: {0}")]
    Measure(#[from] bijux_core::measure::MeasureError),
    #[error("validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ImageQaOutcome {
    Pass,
    Fail(String),
}

impl ImageQaOutcome {
    #[must_use]
    pub fn status(&self) -> &'static str {
        match self {
            ImageQaOutcome::Pass => "pass",
            ImageQaOutcome::Fail(_) => "fail",
        }
    }

    #[must_use]
    pub fn failure_reason(&self) -> Option<&str> {
        match self {
            ImageQaOutcome::Pass => None,
            ImageQaOutcome::Fail(reason) => Some(reason.as_str()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageQaRecord {
    pub tool: String,
    pub stage: String,
    pub tool_version: String,
    pub image_digest: String,
    pub runner: String,
    pub platform: String,
    pub input_hash: String,
    pub outcome: ImageQaOutcome,
}

/// Append a benchmark record as a JSONL line.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn append_jsonl<T>(path: &Path, record: &BenchmarkRecord<T>) -> Result<()>
where
    T: StageMetricSchema + Serialize,
{
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(record)?;
    writeln!(file, "{line}")?;
    Ok(())
}

/// Append an image QA record as a JSONL line.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn append_image_qa_jsonl(path: &Path, record: &ImageQaRecord) -> Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(record)?;
    writeln!(file, "{line}")?;
    Ok(())
}

pub const FASTQ_TRIM_SCHEMA_VERSION: i32 = 2;
pub const FASTQ_VALIDATE_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_FILTER_SCHEMA_VERSION: i32 = 2;
pub const FASTQ_MERGE_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_CORRECT_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_QC_POST_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_UMI_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_SCREEN_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_STATS_SCHEMA_VERSION: i32 = 1;
pub const IMAGE_QA_SCHEMA_VERSION: i32 = 1;
pub const IMAGE_QA_INPUTS_SCHEMA_VERSION: i32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqValidateMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub reads_total: u64,
    pub reads_valid: u64,
    pub reads_invalid: u64,
    pub mean_q: f64,
}

impl StageMetricSchema for FastqValidateMetrics {
    const STAGE: &'static str = "fastq.validate_pre";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_valid + self.reads_invalid != self.reads_total {
            return Err(BenchError::Validation(
                "reads_valid + reads_invalid must equal reads_total".to_string(),
            ));
        }
        if !self.mean_q.is_finite() || !(0.0..=45.0).contains(&self.mean_q) {
            return Err(BenchError::Validation(
                "mean_q must be within [0, 45]".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqFilterMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub reads_dropped: u64,
    #[serde(default)]
    pub reads_removed_by_n: u64,
    #[serde(default)]
    pub reads_removed_by_entropy: u64,
    #[serde(default)]
    pub reads_removed_by_kmer: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    #[serde(default)]
    pub delta_metrics: FastqDeltaMetrics,
}

impl StageMetricSchema for FastqFilterMetrics {
    const STAGE: &'static str = "fastq.filter";
    const VERSION: i32 = 2;

    fn validate(&self) -> Result<()> {
        if self.reads_out + self.reads_dropped != self.reads_in {
            return Err(BenchError::Validation(
                "reads_out + reads_dropped must equal reads_in".to_string(),
            ));
        }
        let removed_breakdown =
            self.reads_removed_by_n + self.reads_removed_by_entropy + self.reads_removed_by_kmer;
        if removed_breakdown > self.reads_dropped {
            return Err(BenchError::Validation(
                "reads_removed_by_* must be <= reads_dropped".to_string(),
            ));
        }
        if self.mean_q_after < self.mean_q_before {
            warn!(
                mean_q_before = self.mean_q_before,
                mean_q_after = self.mean_q_after,
                "mean_q_after is lower than mean_q_before"
            );
        }
        self.delta_metrics.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqMergeMetrics {
    #[serde(default)]
    pub reads_in: u64,
    #[serde(default)]
    pub reads_out: u64,
    #[serde(default)]
    pub bases_in: u64,
    #[serde(default)]
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: u64,
    #[serde(default)]
    pub pairs_out: u64,
    pub reads_r1: u64,
    pub reads_r2: u64,
    pub reads_merged: u64,
    pub reads_unmerged: u64,
    pub merge_rate: f64,
}

impl StageMetricSchema for FastqMergeMetrics {
    const STAGE: &'static str = "fastq.merge";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        let min_reads = self.reads_r1.min(self.reads_r2);
        if self.reads_merged + self.reads_unmerged > min_reads {
            return Err(BenchError::Validation(
                "reads_merged + reads_unmerged must be <= min(reads_r1, reads_r2)".to_string(),
            ));
        }
        if !self.merge_rate.is_finite() || !(0.0..=1.0).contains(&self.merge_rate) {
            return Err(BenchError::Validation(
                "merge_rate must be within [0, 1]".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqCorrectMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    pub kmer_fix_rate: f64,
}

impl StageMetricSchema for FastqCorrectMetrics {
    const STAGE: &'static str = "fastq.correct";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out != self.reads_in {
            return Err(BenchError::Validation(
                "reads_out must equal reads_in".to_string(),
            ));
        }
        if self.bases_out > self.bases_in {
            return Err(BenchError::Validation(
                "bases_out must be <= bases_in".to_string(),
            ));
        }
        if self.mean_q_after < self.mean_q_before {
            warn!(
                mean_q_before = self.mean_q_before,
                mean_q_after = self.mean_q_after,
                "mean_q_after is lower than mean_q_before"
            );
        }
        if !self.kmer_fix_rate.is_finite() || !(0.0..=1.0).contains(&self.kmer_fix_rate) {
            return Err(BenchError::Validation(
                "kmer_fix_rate must be within [0, 1]".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqQcPostMetrics {
    pub reads_in: u64,
    pub bases_in: u64,
    pub mean_q: f64,
    pub contamination_rate: f64,
}

impl StageMetricSchema for FastqQcPostMetrics {
    const STAGE: &'static str = "fastq.qc_post";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if !self.mean_q.is_finite() || !(0.0..=45.0).contains(&self.mean_q) {
            return Err(BenchError::Validation(
                "mean_q must be within [0, 45]".to_string(),
            ));
        }
        if !self.contamination_rate.is_finite() || !(0.0..=1.0).contains(&self.contamination_rate) {
            return Err(BenchError::Validation(
                "contamination_rate must be within [0, 1]".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqUmiMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    #[serde(default)]
    pub bases_in: u64,
    #[serde(default)]
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub dedup_rate: f64,
}

impl StageMetricSchema for FastqUmiMetrics {
    const STAGE: &'static str = "fastq.umi";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation(
                "reads_out must be <= reads_in".to_string(),
            ));
        }
        if !self.dedup_rate.is_finite() || !(0.0..=1.0).contains(&self.dedup_rate) {
            return Err(BenchError::Validation(
                "dedup_rate must be within [0, 1]".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqScreenMetrics {
    pub reads_in: u64,
    pub contamination_rate: f64,
}

impl StageMetricSchema for FastqScreenMetrics {
    const STAGE: &'static str = "fastq.screen";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if !self.contamination_rate.is_finite() || !(0.0..=1.0).contains(&self.contamination_rate) {
            return Err(BenchError::Validation(
                "contamination_rate must be within [0, 1]".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LengthHistogramBin {
    pub length: u64,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqStatsMetrics {
    pub reads_total: u64,
    pub bases_total: u64,
    pub mean_q: f64,
    pub gc_percent: f64,
    pub length_histogram: Vec<LengthHistogramBin>,
}

impl StageMetricSchema for FastqStatsMetrics {
    const STAGE: &'static str = "fastq.stats_neutral";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if !self.mean_q.is_finite() || !(0.0..=45.0).contains(&self.mean_q) {
            return Err(BenchError::Validation(
                "mean_q must be within [0, 45]".to_string(),
            ));
        }
        if !self.gc_percent.is_finite() || !(0.0..=100.0).contains(&self.gc_percent) {
            return Err(BenchError::Validation(
                "gc_percent must be within [0, 100]".to_string(),
            ));
        }
        Ok(())
    }
}
/// Open a `SQLite` connection for benchmark persistence.
///
/// # Errors
/// Returns an error if the connection cannot be opened.
pub struct RankingInput<T: StageMetricSchema> {
    pub stage: StageMetricKind,
    pub execution: ExecutionMetrics,
    pub metrics: T,
}

#[must_use]
pub fn normalize_rate(value: f64) -> Option<f64> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Some(value)
    } else {
        None
    }
}

#[must_use]
pub fn normalize_inverse_rate(value: f64) -> Option<f64> {
    normalize_rate(value).map(|v| 1.0 - v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_serializes() -> Result<()> {
        let record = BenchmarkRecord {
            context: BenchmarkContext {
                tool: "fastp".to_string(),
                tool_version: "0.23.4".to_string(),
                image_digest: "sha256:abc".to_string(),
                runner: "docker".to_string(),
                platform: "local".to_string(),
                input_hash: "sha256:deadbeef".to_string(),
                parameters: serde_json::json!({"adapter": "AGAT"}),
            },
            execution: ExecutionMetrics {
                runtime_s: 1.0,
                memory_mb: 10.0,
                exit_code: 0,
            },
            metrics: metric_set(FastqTrimMetrics {
                reads_in: 100,
                reads_out: 90,
                bases_in: 1000,
                bases_out: 900,
                pairs_in: None,
                pairs_out: None,
                mean_q_before: 30.0,
                mean_q_after: 31.0,
                delta_metrics: FastqDeltaMetrics {
                    read_retention: 0.9,
                    base_retention: 0.9,
                    mean_q_delta: 1.0,
                    gc_delta: 0.1,
                },
                adapter_preset: None,
                adapter_bank_id: None,
                adapter_bank_hash: None,
                adapter_overrides: None,
            }),
        };
        record.validate()?;
        let json = serde_json::to_string(&record)?;
        assert!(json.contains("fastp"));
        assert!(json.contains("metrics_schema"));
        Ok(())
    }

    #[test]
    fn filter_metrics_invariants() -> Result<()> {
        let metrics = FastqFilterMetrics {
            reads_in: 100,
            reads_out: 80,
            reads_dropped: 20,
            reads_removed_by_n: 10,
            reads_removed_by_entropy: 5,
            reads_removed_by_kmer: 5,
            bases_in: 1000,
            bases_out: 800,
            pairs_in: None,
            pairs_out: None,
            mean_q_before: 30.0,
            mean_q_after: 29.0,
            delta_metrics: FastqDeltaMetrics {
                read_retention: 0.8,
                base_retention: 0.8,
                mean_q_delta: -1.0,
                gc_delta: 0.0,
            },
        };
        metrics.validate()?;
        let invalid = FastqFilterMetrics {
            reads_in: 100,
            reads_out: 81,
            reads_dropped: 20,
            reads_removed_by_n: 10,
            reads_removed_by_entropy: 10,
            reads_removed_by_kmer: 5,
            bases_in: 1000,
            bases_out: 810,
            pairs_in: None,
            pairs_out: None,
            mean_q_before: 30.0,
            mean_q_after: 31.0,
            delta_metrics: FastqDeltaMetrics {
                read_retention: 0.81,
                base_retention: 0.81,
                mean_q_delta: 1.0,
                gc_delta: 0.0,
            },
        };
        assert!(invalid.validate().is_err());
        Ok(())
    }

    #[test]
    fn merge_metrics_invariants() -> Result<()> {
        let metrics = FastqMergeMetrics {
            reads_in: 100,
            reads_out: 60,
            bases_in: 1000,
            bases_out: 600,
            pairs_in: 100,
            pairs_out: 60,
            reads_r1: 100,
            reads_r2: 120,
            reads_merged: 60,
            reads_unmerged: 40,
            merge_rate: 0.6,
        };
        metrics.validate()?;
        let invalid = FastqMergeMetrics {
            reads_in: 100,
            reads_out: 80,
            bases_in: 1000,
            bases_out: 800,
            pairs_in: 100,
            pairs_out: 80,
            reads_r1: 100,
            reads_r2: 100,
            reads_merged: 80,
            reads_unmerged: 30,
            merge_rate: 1.2,
        };
        assert!(invalid.validate().is_err());
        Ok(())
    }
}
