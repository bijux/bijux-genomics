//! Owner: bijux-analyze
//! Metric registry definitions and stage metric sets.

use serde::{Deserialize, Serialize};

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
    ReadsRemovedLowComplexity,
    ReadsRemovedByKmer,
    ReadsRemovedContaminantKmer,
    ReadsRemovedByLength,
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
    ContaminationSummary,
    GcPercent,
    LengthHistogram,
    DeltaMetrics,
    AdapterPreset,
    AdapterBankId,
    AdapterBankHash,
    AdapterOverrides,
    QcRawDir,
    QcTrimmedDir,
    MultiqcReport,
    MultiqcData,
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
    pub version: i32,
    pub metrics: &'static [MetricId],
    pub invariants: &'static [&'static str],
}

pub const METRIC_REGISTRY_CORE: [MetricSpec; 15] = [
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
        meaning: "Exit code from tool execution",
        direction: MetricDirection::Neutral,
        range: None,
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
            "fastq.qc_post",
            "fastq.umi",
            "fastq.screen",
            "fastq.stats_neutral",
        ],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ReadsOut,
        name: "reads_out",
        meaning: "Number of output reads",
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
            "fastq.screen",
        ],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ReadsDropped,
        name: "reads_dropped",
        meaning: "Number of reads dropped during filtering",
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
        meaning: "Reads removed due to low entropy",
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
        id: MetricId::ReadsRemovedLowComplexity,
        name: "reads_removed_low_complexity",
        meaning: "Reads removed due to low complexity",
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
        meaning: "Reads removed due to k-mer matching",
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
        id: MetricId::ReadsRemovedContaminantKmer,
        name: "reads_removed_contaminant_kmer",
        meaning: "Reads removed due to contaminant k-mer matching",
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
        id: MetricId::ReadsRemovedByLength,
        name: "reads_removed_by_length",
        meaning: "Reads removed due to length filtering",
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
        meaning: "Total reads for validation",
        direction: MetricDirection::Neutral,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &["fastq.validate_pre", "fastq.stats_neutral"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ReadsValid,
        name: "reads_valid",
        meaning: "Reads that pass validation",
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
        meaning: "Reads that fail validation",
        direction: MetricDirection::LowerBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: f64::INFINITY,
        }),
        stages: &["fastq.validate_pre"],
        measured: true,
        derived: false,
    },
];

pub const METRIC_REGISTRY_QUALITY: [MetricSpec; 15] = [
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
            "fastq.screen",
        ],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::BasesOut,
        name: "bases_out",
        meaning: "Number of output bases",
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
            "fastq.screen",
        ],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::BasesTotal,
        name: "bases_total",
        meaning: "Total bases for stats",
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
            "fastq.qc_post",
            "fastq.umi",
            "fastq.screen",
        ],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::PairsOut,
        name: "pairs_out",
        meaning: "Number of output read pairs",
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
            "fastq.screen",
        ],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::ReadsR1,
        name: "reads_r1",
        meaning: "Number of reads in R1",
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
        meaning: "Number of reads in R2",
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
        meaning: "Number of unmerged reads",
        direction: MetricDirection::LowerBetter,
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
        meaning: "Mean quality before processing",
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
        meaning: "Mean quality after processing",
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
        meaning: "Mean quality score",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange {
            min: 0.0,
            max: 45.0,
        }),
        stages: &["fastq.validate_pre", "fastq.qc_post", "fastq.stats_neutral"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::MergeRate,
        name: "merge_rate",
        meaning: "Merged reads / total reads",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange { min: 0.0, max: 1.0 }),
        stages: &["fastq.merge"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::DedupRate,
        name: "dedup_rate",
        meaning: "Fraction of reads retained after deduplication",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange { min: 0.0, max: 1.0 }),
        stages: &["fastq.umi"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::KmerFixRate,
        name: "kmer_fix_rate",
        meaning: "Fraction of k-mers corrected",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange { min: 0.0, max: 1.0 }),
        stages: &["fastq.correct"],
        measured: true,
        derived: false,
    },
];

pub const METRIC_REGISTRY_FASTQ: [MetricSpec; 13] = [
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
        id: MetricId::ContaminationSummary,
        name: "contamination_summary",
        meaning: "Structured contamination summary",
        direction: MetricDirection::Neutral,
        range: None,
        stages: &["fastq.screen"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::GcPercent,
        name: "gc_percent",
        meaning: "GC percent",
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
        meaning: "Read length histogram",
        direction: MetricDirection::Neutral,
        range: None,
        stages: &["fastq.stats_neutral"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::DeltaMetrics,
        name: "delta_metrics",
        meaning: "Read/base retention deltas",
        direction: MetricDirection::HigherBetter,
        range: None,
        stages: &["fastq.trim", "fastq.filter", "fastq.correct"],
        measured: false,
        derived: true,
    },
    MetricSpec {
        id: MetricId::AdapterPreset,
        name: "adapter_preset",
        meaning: "Adapter preset identifier",
        direction: MetricDirection::Neutral,
        range: None,
        stages: &["fastq.trim"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::AdapterBankId,
        name: "adapter_bank_id",
        meaning: "Adapter bank identifier",
        direction: MetricDirection::Neutral,
        range: None,
        stages: &["fastq.trim"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::AdapterBankHash,
        name: "adapter_bank_hash",
        meaning: "Adapter bank hash",
        direction: MetricDirection::Neutral,
        range: None,
        stages: &["fastq.trim"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::AdapterOverrides,
        name: "adapter_overrides",
        meaning: "Adapter override parameters",
        direction: MetricDirection::Neutral,
        range: None,
        stages: &["fastq.trim"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::QcRawDir,
        name: "raw_fastqc_dir",
        meaning: "Raw FastQC directory",
        direction: MetricDirection::Neutral,
        range: None,
        stages: &["fastq.qc_post"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::QcTrimmedDir,
        name: "trimmed_fastqc_dir",
        meaning: "Trimmed FastQC directory",
        direction: MetricDirection::Neutral,
        range: None,
        stages: &["fastq.qc_post"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::MultiqcReport,
        name: "multiqc_report",
        meaning: "MultiQC report path",
        direction: MetricDirection::Neutral,
        range: None,
        stages: &["fastq.qc_post"],
        measured: true,
        derived: false,
    },
    MetricSpec {
        id: MetricId::MultiqcData,
        name: "multiqc_data",
        meaning: "MultiQC data directory",
        direction: MetricDirection::Neutral,
        range: None,
        stages: &["fastq.qc_post"],
        measured: true,
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
        stages: &["fastq.trim", "fastq.filter", "fastq.correct", "fastq.umi"],
    },
    DerivedMetricSpec {
        id: DerivedMetricId::MergeEfficiency,
        name: "merge_efficiency",
        meaning: "reads_merged / (reads_merged + reads_unmerged)",
        direction: MetricDirection::HigherBetter,
        range: Some(MetricRange { min: 0.0, max: 1.0 }),
        stages: &["fastq.merge"],
    },
    DerivedMetricSpec {
        id: DerivedMetricId::ErrorReductionProxy,
        name: "error_reduction_proxy",
        meaning: "proxy for error reduction",
        direction: MetricDirection::HigherBetter,
        range: None,
        stages: &["fastq.correct"],
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

pub const FASTQ_FILTER_METRICS: [MetricId; 16] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::ReadsDropped,
    MetricId::ReadsRemovedByN,
    MetricId::ReadsRemovedByEntropy,
    MetricId::ReadsRemovedLowComplexity,
    MetricId::ReadsRemovedByKmer,
    MetricId::ReadsRemovedContaminantKmer,
    MetricId::ReadsRemovedByLength,
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

pub const FASTQ_QC_POST_METRICS: [MetricId; 12] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::PairsIn,
    MetricId::PairsOut,
    MetricId::MeanQ,
    MetricId::ContaminationRate,
    MetricId::QcRawDir,
    MetricId::QcTrimmedDir,
    MetricId::MultiqcReport,
    MetricId::MultiqcData,
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

pub const FASTQ_SCREEN_METRICS: [MetricId; 8] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::PairsIn,
    MetricId::PairsOut,
    MetricId::ContaminationRate,
    MetricId::ContaminationSummary,
];

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

pub const FASTQ_SCREEN_INVARIANTS: [&str; 4] = [
    "reads_out <= reads_in",
    "bases_out <= bases_in",
    "contamination_rate in [0, 1]",
    "counts are non-negative",
];

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
