pub const FASTQ_TRIM_METRICS: [MetricId; 19] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::PairsIn,
    MetricId::PairsOut,
    MetricId::MeanQBefore,
    MetricId::MeanQAfter,
    MetricId::DeltaMetrics,
    MetricId::PairedMode,
    MetricId::AdapterPolicy,
    MetricId::PolyxPolicy,
    MetricId::NPolicy,
    MetricId::ContaminantPolicy,
    MetricId::RawBackendReportFormat,
    MetricId::AdapterPreset,
    MetricId::AdapterBankId,
    MetricId::AdapterBankHash,
    MetricId::AdapterOverrides,
];

pub const FASTQ_TRIM_POLYG_METRICS: [MetricId; 18] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::PairsIn,
    MetricId::PairsOut,
    MetricId::MeanQBefore,
    MetricId::MeanQAfter,
    MetricId::DeltaMetrics,
    MetricId::PairedMode,
    MetricId::Threads,
    MetricId::TrimPolyg,
    MetricId::MinPolygRun,
    MetricId::BasesTrimmedPolyg,
    MetricId::RawBackendReportFormat,
    MetricId::PolyxBankId,
    MetricId::PolyxBankHash,
    MetricId::PolyxPreset,
];

pub const FASTQ_TRIM_TERMINAL_DAMAGE_METRICS: [MetricId; 16] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::PairsIn,
    MetricId::PairsOut,
    MetricId::MeanQBefore,
    MetricId::MeanQAfter,
    MetricId::DamageMode,
    MetricId::ExecutionPolicy,
    MetricId::RequestedTrim5pBases,
    MetricId::RequestedTrim3pBases,
    MetricId::UdgClassification,
    MetricId::CtGaAsymmetryPre,
    MetricId::CtGaAsymmetryPost,
    MetricId::DeltaMetrics,
];

pub const FASTQ_VALIDATE_METRICS: [MetricId; 17] = [
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
    MetricId::ValidatedInputs,
    MetricId::ValidatedPairs,
    MetricId::PairSyncChecked,
    MetricId::PairSyncPass,
    MetricId::PairCountMatch,
    MetricId::StrictPass,
    MetricId::FailureClass,
];

pub const FASTQ_DETECT_ADAPTERS_METRICS: [MetricId; 7] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::MeanQ,
    MetricId::CandidateAdapterCount,
    MetricId::AdapterTrimmedFraction,
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

pub const FASTQ_LOW_COMPLEXITY_METRICS: [MetricId; 8] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::ReadsRemovedLowComplexity,
    MetricId::MeanQBefore,
    MetricId::MeanQAfter,
    MetricId::DeltaMetrics,
];

pub const FASTQ_DEDUPLICATE_METRICS: [MetricId; 12] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::DuplicateReads,
    MetricId::DedupRate,
    MetricId::Tool,
    MetricId::PairedMode,
    MetricId::DedupMode,
    MetricId::KeepOrder,
    MetricId::PairCountMatch,
    MetricId::DuplicateClassCount,
    MetricId::DuplicateProvenanceJson,
    MetricId::RawBackendReportFormat,
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

pub const FASTQ_QC_POST_METRICS: [MetricId; 20] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::PairsIn,
    MetricId::PairsOut,
    MetricId::MeanQ,
    MetricId::ContaminationRate,
    MetricId::AggregationEngine,
    MetricId::AggregationScope,
    MetricId::GovernedQcInputCount,
    MetricId::GovernedQcContributorStageIds,
    MetricId::GovernedQcContributorToolIds,
    MetricId::GovernedQcLineageHash,
    MetricId::MultiqcSampleCount,
    MetricId::MultiqcModuleCount,
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
    MetricId::ReadsWithUmi,
];

pub const FASTQ_INDEX_REFERENCE_METRICS: [MetricId; 3] =
    [MetricId::ReferenceBytes, MetricId::IndexBytes, MetricId::IndexFileCount];

pub const FASTQ_DEPLETE_HOST_METRICS: [MetricId; 8] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::PairsIn,
    MetricId::PairsOut,
    MetricId::HostFractionRemoved,
    MetricId::DepletionSummary,
];

pub const FASTQ_DEPLETE_REFERENCE_CONTAMINANTS_METRICS: [MetricId; 8] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::PairsIn,
    MetricId::PairsOut,
    MetricId::ContaminantFractionRemoved,
    MetricId::DepletionSummary,
];

pub const FASTQ_DEPLETE_RRNA_METRICS: [MetricId; 8] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::PairsIn,
    MetricId::PairsOut,
    MetricId::RrnaFractionRemoved,
    MetricId::DepletionSummary,
];

pub const FASTQ_SCREEN_METRICS: [MetricId; 17] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::BasesIn,
    MetricId::BasesOut,
    MetricId::PairsIn,
    MetricId::PairsOut,
    MetricId::ContaminationRate,
    MetricId::ClassifiedFraction,
    MetricId::UnclassifiedFraction,
    MetricId::Classifier,
    MetricId::ReportFormat,
    MetricId::DatabaseCatalogId,
    MetricId::DatabaseArtifactId,
    MetricId::MinimumConfidence,
    MetricId::EmitUnclassified,
    MetricId::ContaminationSummary,
    MetricId::TopTaxa,
];

pub const FASTQ_NORMALIZE_PRIMERS_METRICS: [MetricId; 4] = [
    MetricId::ReadsIn,
    MetricId::ReadsOut,
    MetricId::PrimerTrimmedFraction,
    MetricId::OrientationForwardFraction,
];

pub const FASTQ_STATS_METRICS: [MetricId; 5] = [
    MetricId::ReadsTotal,
    MetricId::BasesTotal,
    MetricId::MeanQ,
    MetricId::GcPercent,
    MetricId::LengthHistogram,
];

pub const FASTQ_READ_LENGTH_METRICS: [MetricId; 6] = [
    MetricId::ReadCount,
    MetricId::MinReadLength,
    MetricId::MeanReadLength,
    MetricId::MedianReadLength,
    MetricId::MaxReadLength,
    MetricId::DistinctLengths,
];

pub const FASTQ_OVERREPRESENTED_METRICS: [MetricId; 3] =
    [MetricId::SequenceCount, MetricId::FlaggedSequences, MetricId::TopFraction];

pub const FASTQ_TRIM_INVARIANTS: [&str; 4] = [
    "reads_out <= reads_in",
    "bases_out <= bases_in",
    "mean_q_after >= mean_q_before (warn)",
    "counts are non-negative",
];

pub const FASTQ_VALIDATE_INVARIANTS: [&str; 3] =
    ["reads_valid + reads_invalid == reads_total", "mean_q in [0, 45]", "counts are non-negative"];

pub const FASTQ_DETECT_ADAPTERS_INVARIANTS: [&str; 4] = [
    "reads_out == reads_in",
    "bases_out == bases_in",
    "mean_q in [0, 45]",
    "adapter_trimmed_fraction in [0, 1]",
];

pub const FASTQ_FILTER_INVARIANTS: [&str; 3] = [
    "reads_out + reads_dropped == reads_in",
    "mean_q_after >= mean_q_before (warn)",
    "counts are non-negative",
];

pub const FASTQ_LOW_COMPLEXITY_INVARIANTS: [&str; 4] = [
    "reads_out + reads_removed_low_complexity == reads_in",
    "bases_out <= bases_in",
    "mean_q_after >= mean_q_before (warn)",
    "counts are non-negative",
];

pub const FASTQ_DEDUPLICATE_INVARIANTS: [&str; 4] = [
    "reads_out <= reads_in",
    "dedup_rate in [0, 1]",
    "pair_count_match should not be false for paired inputs",
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

pub const FASTQ_QC_POST_INVARIANTS: [&str; 3] =
    ["mean_q in [0, 45]", "contamination_rate in [0, 1]", "counts are non-negative"];

pub const FASTQ_UMI_INVARIANTS: [&str; 3] =
    ["reads_out <= reads_in", "reads_with_umi <= reads_out", "counts are non-negative"];

pub const FASTQ_INDEX_REFERENCE_INVARIANTS: [&str; 3] =
    ["reference_bytes > 0", "index_file_count > 0", "index_bytes >= index_file_count"];

pub const FASTQ_DEPLETE_HOST_INVARIANTS: [&str; 4] = [
    "reads_out <= reads_in",
    "bases_out <= bases_in",
    "host_fraction_removed in [0, 1]",
    "counts are non-negative",
];

pub const FASTQ_DEPLETE_REFERENCE_CONTAMINANTS_INVARIANTS: [&str; 4] = [
    "reads_out <= reads_in",
    "bases_out <= bases_in",
    "contaminant_fraction_removed in [0, 1]",
    "counts are non-negative",
];

pub const FASTQ_DEPLETE_RRNA_INVARIANTS: [&str; 4] = [
    "reads_out <= reads_in",
    "bases_out <= bases_in",
    "rrna_fraction_removed in [0, 1]",
    "counts are non-negative",
];

pub const FASTQ_SCREEN_INVARIANTS: [&str; 4] = [
    "reads_out <= reads_in",
    "bases_out <= bases_in",
    "contamination_rate in [0, 1]",
    "counts are non-negative",
];

pub const FASTQ_NORMALIZE_PRIMERS_INVARIANTS: [&str; 4] = [
    "reads_out <= reads_in",
    "primer_trimmed_fraction in [0, 1]",
    "orientation_forward_fraction in [0, 1]",
    "counts are non-negative",
];

pub const FASTQ_STATS_INVARIANTS: [&str; 2] = ["mean_q in [0, 45]", "gc_percent in [0, 100]"];

pub const FASTQ_READ_LENGTH_INVARIANTS: [&str; 6] = [
    "mean_read_length >= 0",
    "min_read_length > 0 when reads are present",
    "median_read_length >= min_read_length when reads are present",
    "max_read_length > 0 when reads are present",
    "median_read_length <= max_read_length when reads are present",
    "distinct_lengths <= read_count",
];

pub const FASTQ_OVERREPRESENTED_INVARIANTS: [&str; 2] =
    ["flagged_sequences <= sequence_count", "top_fraction in [0, 1]"];

#[must_use]
pub fn metric_kind_for_stage(stage_id: &str) -> Option<StageMetricKind> {
    match stage_id {
        "fastq.trim_reads" => Some(StageMetricKind::FastqTrim),
        "fastq.trim_polyg_tails" => Some(StageMetricKind::FastqTrimPolyg),
        "fastq.trim_terminal_damage" => Some(StageMetricKind::FastqTrimTerminalDamage),
        "fastq.validate_reads" => Some(StageMetricKind::FastqValidate),
        "fastq.detect_adapters" => Some(StageMetricKind::FastqDetectAdapters),
        "fastq.filter_reads" => Some(StageMetricKind::FastqFilter),
        "fastq.filter_low_complexity" => Some(StageMetricKind::FastqLowComplexity),
        "fastq.remove_duplicates" => Some(StageMetricKind::FastqDeduplicate),
        "fastq.merge_pairs" => Some(StageMetricKind::FastqMerge),
        "fastq.correct_errors" => Some(StageMetricKind::FastqCorrect),
        "fastq.report_qc" => Some(StageMetricKind::FastqQcPost),
        "fastq.extract_umis" => Some(StageMetricKind::FastqUmi),
        "fastq.index_reference" => Some(StageMetricKind::FastqIndexReference),
        "fastq.deplete_host" => Some(StageMetricKind::FastqDepleteHost),
        "fastq.deplete_reference_contaminants" => {
            Some(StageMetricKind::FastqDepleteReferenceContaminants)
        }
        "fastq.deplete_rrna" => Some(StageMetricKind::FastqDepleteRrna),
        "fastq.screen_taxonomy" => Some(StageMetricKind::FastqScreen),
        "fastq.normalize_primers" => Some(StageMetricKind::FastqNormalizePrimers),
        "fastq.profile_reads" => Some(StageMetricKind::FastqStats),
        "fastq.profile_read_lengths" => Some(StageMetricKind::FastqReadLengths),
        "fastq.profile_overrepresented_sequences" => Some(StageMetricKind::FastqOverrepresented),
        _ => None,
    }
}
use crate::aggregate::schema::defs::{MetricId, StageMetricKind};
