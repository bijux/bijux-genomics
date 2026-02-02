#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum MetricClass {
    Integrity,
    Retention,
    QualityShift,
    Contamination,
    Composition,
}

#[derive(Debug, Clone, Copy)]
pub struct StageMetricSpec {
    pub stage: &'static str,
    pub classes: &'static [MetricClass],
    pub invariants: &'static [&'static str],
    pub notes: &'static str,
}

pub const FASTQ_VALIDATE_CLASSES: [MetricClass; 1] = [MetricClass::Integrity];
pub const FASTQ_DETECT_CLASSES: [MetricClass; 1] = [MetricClass::Composition];
pub const FASTQ_TRIM_CLASSES: [MetricClass; 3] = [
    MetricClass::Integrity,
    MetricClass::Retention,
    MetricClass::QualityShift,
];
pub const FASTQ_FILTER_CLASSES: [MetricClass; 3] = [
    MetricClass::Integrity,
    MetricClass::Retention,
    MetricClass::QualityShift,
];
pub const FASTQ_MERGE_CLASSES: [MetricClass; 2] = [MetricClass::Integrity, MetricClass::Retention];
pub const FASTQ_CORRECT_CLASSES: [MetricClass; 2] =
    [MetricClass::Integrity, MetricClass::QualityShift];
pub const FASTQ_UMI_CLASSES: [MetricClass; 2] = [MetricClass::Integrity, MetricClass::Retention];
pub const FASTQ_SCREEN_CLASSES: [MetricClass; 1] = [MetricClass::Contamination];
pub const FASTQ_QC_POST_CLASSES: [MetricClass; 2] =
    [MetricClass::QualityShift, MetricClass::Contamination];
pub const FASTQ_STATS_CLASSES: [MetricClass; 2] =
    [MetricClass::Integrity, MetricClass::Composition];

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
pub const FASTQ_DETECT_INVARIANTS: [&str; 2] =
    ["counts are non-negative", "adapter_content in [0, 100]"];

pub const FASTQ_FILTER_INVARIANTS: [&str; 4] = [
    "reads_out + reads_dropped == reads_in",
    "reads_removed_by_* <= reads_dropped",
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

pub const FASTQ_QC_POST_INVARIANTS: [&str; 5] = [
    "reads_out <= reads_in",
    "bases_out <= bases_in",
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
pub fn metric_spec_for_stage(stage_id: &str) -> Option<StageMetricSpec> {
    match stage_id {
        "fastq.validate_pre" => Some(StageMetricSpec {
            stage: "fastq.validate_pre",
            classes: &FASTQ_VALIDATE_CLASSES,
            invariants: &FASTQ_VALIDATE_INVARIANTS,
            notes: "Validation reports counts; no data is modified.",
        }),
        "fastq.detect_adapters" => Some(StageMetricSpec {
            stage: "fastq.detect_adapters",
            classes: &FASTQ_DETECT_CLASSES,
            invariants: &FASTQ_DETECT_INVARIANTS,
            notes: "Adapter detection inspects reads and reports adapter signals.",
        }),
        "fastq.trim" => Some(StageMetricSpec {
            stage: "fastq.trim",
            classes: &FASTQ_TRIM_CLASSES,
            invariants: &FASTQ_TRIM_INVARIANTS,
            notes: "Trim can reduce reads/bases and improve quality.",
        }),
        "fastq.filter" => Some(StageMetricSpec {
            stage: "fastq.filter",
            classes: &FASTQ_FILTER_CLASSES,
            invariants: &FASTQ_FILTER_INVARIANTS,
            notes: "Filter drops reads and should improve quality.",
        }),
        "fastq.merge" => Some(StageMetricSpec {
            stage: "fastq.merge",
            classes: &FASTQ_MERGE_CLASSES,
            invariants: &FASTQ_MERGE_INVARIANTS,
            notes: "Merge produces merged/unmerged reads from pairs.",
        }),
        "fastq.correct" => Some(StageMetricSpec {
            stage: "fastq.correct",
            classes: &FASTQ_CORRECT_CLASSES,
            invariants: &FASTQ_CORRECT_INVARIANTS,
            notes: "Correct should preserve reads while improving quality.",
        }),
        "fastq.qc_post" => Some(StageMetricSpec {
            stage: "fastq.qc_post",
            classes: &FASTQ_QC_POST_CLASSES,
            invariants: &FASTQ_QC_POST_INVARIANTS,
            notes: "Post-QC reports quality and contamination signals.",
        }),
        "fastq.umi" => Some(StageMetricSpec {
            stage: "fastq.umi",
            classes: &FASTQ_UMI_CLASSES,
            invariants: &FASTQ_UMI_INVARIANTS,
            notes: "UMI processing may drop reads during deduplication.",
        }),
        "fastq.screen" => Some(StageMetricSpec {
            stage: "fastq.screen",
            classes: &FASTQ_SCREEN_CLASSES,
            invariants: &FASTQ_SCREEN_INVARIANTS,
            notes: "Screening reports contamination only.",
        }),
        "fastq.stats_neutral" => Some(StageMetricSpec {
            stage: "fastq.stats_neutral",
            classes: &FASTQ_STATS_CLASSES,
            invariants: &FASTQ_STATS_INVARIANTS,
            notes: "Stats is report-only and must not mutate reads.",
        }),
        _ => None,
    }
}
