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
