use super::MetricClass;

#[derive(Debug, Clone, Copy)]
pub struct StageMetricSpec {
    pub stage: &'static str,
    pub classes: &'static [MetricClass],
    pub invariants: &'static [&'static str],
    pub notes: &'static str,
}

pub const FASTQ_VALIDATE_CLASSES: [MetricClass; 1] = [MetricClass::Integrity];
pub const FASTQ_DETECT_CLASSES: [MetricClass; 1] = [MetricClass::Composition];
pub const FASTQ_DAMAGE_AWARE_PRETRIM_CLASSES: [MetricClass; 2] =
    [MetricClass::Integrity, MetricClass::Retention];
pub const FASTQ_POLYG_TRIM_CLASSES: [MetricClass; 3] =
    [MetricClass::Integrity, MetricClass::Retention, MetricClass::QualityShift];
pub const FASTQ_TRIM_CLASSES: [MetricClass; 3] =
    [MetricClass::Integrity, MetricClass::Retention, MetricClass::QualityShift];
pub const FASTQ_FILTER_CLASSES: [MetricClass; 3] =
    [MetricClass::Integrity, MetricClass::Retention, MetricClass::QualityShift];
pub const FASTQ_DEDUPLICATE_CLASSES: [MetricClass; 3] =
    [MetricClass::Integrity, MetricClass::Retention, MetricClass::QualityShift];
pub const FASTQ_MERGE_CLASSES: [MetricClass; 2] = [MetricClass::Integrity, MetricClass::Retention];
pub const FASTQ_CORRECT_CLASSES: [MetricClass; 2] =
    [MetricClass::Integrity, MetricClass::QualityShift];
pub const FASTQ_UMI_CLASSES: [MetricClass; 2] = [MetricClass::Integrity, MetricClass::Retention];
pub const FASTQ_SCREEN_CLASSES: [MetricClass; 1] = [MetricClass::Contamination];
pub const FASTQ_RRNA_CLASSES: [MetricClass; 2] =
    [MetricClass::Contamination, MetricClass::Retention];
pub const FASTQ_QC_POST_CLASSES: [MetricClass; 2] =
    [MetricClass::QualityShift, MetricClass::Contamination];
pub const FASTQ_STATS_CLASSES: [MetricClass; 2] =
    [MetricClass::Integrity, MetricClass::Composition];
pub const FASTQ_READ_LENGTH_CLASSES: [MetricClass; 2] =
    [MetricClass::Integrity, MetricClass::Composition];
pub const FASTQ_CHIMERA_CLASSES: [MetricClass; 2] =
    [MetricClass::Integrity, MetricClass::Retention];
pub const FASTQ_CLUSTER_OTUS_CLASSES: [MetricClass; 2] =
    [MetricClass::Integrity, MetricClass::Composition];
pub const FASTQ_NORMALIZE_PRIMERS_CLASSES: [MetricClass; 2] =
    [MetricClass::Integrity, MetricClass::Retention];
pub const FASTQ_INFER_ASVS_CLASSES: [MetricClass; 2] =
    [MetricClass::Integrity, MetricClass::Composition];
pub const FASTQ_NORMALIZE_ABUNDANCE_CLASSES: [MetricClass; 1] = [MetricClass::Composition];
pub const FASTQ_OVERREPRESENTED_CLASSES: [MetricClass; 1] = [MetricClass::Composition];

pub const FASTQ_TRIM_INVARIANTS: [&str; 4] = [
    "reads_out <= reads_in",
    "bases_out <= bases_in",
    "mean_q_after >= mean_q_before (warn)",
    "counts are non-negative",
];

pub const FASTQ_VALIDATE_INVARIANTS: [&str; 3] =
    ["reads_valid + reads_invalid == reads_total", "mean_q in [0, 45]", "counts are non-negative"];
pub const FASTQ_DETECT_INVARIANTS: [&str; 2] =
    ["counts are non-negative", "adapter_content in [0, 100]"];
pub const FASTQ_DAMAGE_AWARE_PRETRIM_INVARIANTS: [&str; 3] =
    ["reads_out <= reads_in", "bases_out <= bases_in", "counts are non-negative"];
pub const FASTQ_POLYG_TRIM_INVARIANTS: [&str; 4] = [
    "reads_out <= reads_in",
    "bases_out <= bases_in",
    "mean_q_after >= mean_q_before (warn)",
    "counts are non-negative",
];

pub const FASTQ_FILTER_INVARIANTS: [&str; 4] = [
    "reads_out + reads_dropped == reads_in",
    "reads_removed_by_* <= reads_dropped",
    "mean_q_after >= mean_q_before (warn)",
    "counts are non-negative",
];
pub const FASTQ_DEDUPLICATE_INVARIANTS: [&str; 4] = [
    "reads_out <= reads_in",
    "reads_removed_duplicates == reads_in - reads_out",
    "retention.value in [0, 1]",
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

pub const FASTQ_UMI_INVARIANTS: [&str; 3] =
    ["reads_out <= reads_in", "dedup_rate in [0, 1]", "counts are non-negative"];

pub const FASTQ_SCREEN_INVARIANTS: [&str; 2] =
    ["contamination_rate in [0, 1]", "counts are non-negative"];
pub const FASTQ_RRNA_INVARIANTS: [&str; 3] =
    ["reads_out <= reads_in", "reads_rrna <= reads_in", "counts are non-negative"];

pub const FASTQ_STATS_INVARIANTS: [&str; 2] = ["mean_q in [0, 45]", "gc_percent in [0, 100]"];
pub const FASTQ_READ_LENGTH_INVARIANTS: [&str; 4] = [
    "read_count >= 0",
    "mean_read_length >= 0",
    "max_read_length >= mean_read_length (warn)",
    "distinct_lengths >= 0",
];
pub const FASTQ_CHIMERA_INVARIANTS: [&str; 4] = [
    "reads_out <= reads_in",
    "chimeras_removed == reads_in - reads_out",
    "chimera_fraction in [0, 1]",
    "counts are non-negative",
];
pub const FASTQ_CLUSTER_OTUS_INVARIANTS: [&str; 3] =
    ["reads_in >= 0", "otu_count >= 0", "representative_count >= 0"];
pub const FASTQ_NORMALIZE_PRIMERS_INVARIANTS: [&str; 3] = [
    "reads_out <= reads_in",
    "primer_trimmed_fraction in [0, 1]",
    "orientation_forward_fraction in [0, 1]",
];
pub const FASTQ_INFER_ASVS_INVARIANTS: [&str; 3] =
    ["asv_count >= 0", "sample_count >= 0", "counts are non-negative"];
pub const FASTQ_NORMALIZE_ABUNDANCE_INVARIANTS: [&str; 3] =
    ["table_rows >= 0", "sample_count >= 0", "zero_fraction in [0, 1]"];
pub const FASTQ_OVERREPRESENTED_INVARIANTS: [&str; 3] =
    ["sequence_count >= 0", "top_fraction in [0, 1]", "counts are non-negative"];

pub const FASTQ_STAGE_METRIC_SPECS: [StageMetricSpec; 22] = [
    StageMetricSpec {
        stage: "fastq.validate_reads",
        classes: &FASTQ_VALIDATE_CLASSES,
        invariants: &FASTQ_VALIDATE_INVARIANTS,
        notes: "Validation reports counts; no data is modified.",
    },
    StageMetricSpec {
        stage: "fastq.detect_adapters",
        classes: &FASTQ_DETECT_CLASSES,
        invariants: &FASTQ_DETECT_INVARIANTS,
        notes: "Adapter detection inspects reads and reports adapter signals.",
    },
    StageMetricSpec {
        stage: "fastq.trim_terminal_damage",
        classes: &FASTQ_DAMAGE_AWARE_PRETRIM_CLASSES,
        invariants: &FASTQ_DAMAGE_AWARE_PRETRIM_INVARIANTS,
        notes: "Damage-aware pretrim can mask or trim terminal damage while preserving deterministic output order.",
    },
    StageMetricSpec {
        stage: "fastq.trim_polyg_tails",
        classes: &FASTQ_POLYG_TRIM_CLASSES,
        invariants: &FASTQ_POLYG_TRIM_INVARIANTS,
        notes: "PolyG tail trimming removes sequencer-specific terminal artifacts while preserving deterministic read order.",
    },
    StageMetricSpec {
        stage: "fastq.trim_reads",
        classes: &FASTQ_TRIM_CLASSES,
        invariants: &FASTQ_TRIM_INVARIANTS,
        notes: "Trim can reduce reads/bases and improve quality.",
    },
    StageMetricSpec {
        stage: "fastq.filter_reads",
        classes: &FASTQ_FILTER_CLASSES,
        invariants: &FASTQ_FILTER_INVARIANTS,
        notes: "Filter drops reads and should improve quality.",
    },
    StageMetricSpec {
        stage: "fastq.remove_duplicates",
        classes: &FASTQ_DEDUPLICATE_CLASSES,
        invariants: &FASTQ_DEDUPLICATE_INVARIANTS,
        notes: "Deduplication removes duplicate reads while preserving pair semantics.",
    },
    StageMetricSpec {
        stage: "fastq.filter_low_complexity",
        classes: &FASTQ_FILTER_CLASSES,
        invariants: &FASTQ_FILTER_INVARIANTS,
        notes: "Low-complexity filtering drops reads based on entropy/polyN/polyX signals.",
    },
    StageMetricSpec {
        stage: "fastq.merge_pairs",
        classes: &FASTQ_MERGE_CLASSES,
        invariants: &FASTQ_MERGE_INVARIANTS,
        notes: "Merge produces merged/unmerged reads from pairs.",
    },
    StageMetricSpec {
        stage: "fastq.correct_errors",
        classes: &FASTQ_CORRECT_CLASSES,
        invariants: &FASTQ_CORRECT_INVARIANTS,
        notes: "Correct should preserve reads while improving quality.",
    },
    StageMetricSpec {
        stage: "fastq.report_qc",
        classes: &FASTQ_QC_POST_CLASSES,
        invariants: &FASTQ_QC_POST_INVARIANTS,
        notes: "Post-QC reports quality and contamination signals.",
    },
    StageMetricSpec {
        stage: "fastq.extract_umis",
        classes: &FASTQ_UMI_CLASSES,
        invariants: &FASTQ_UMI_INVARIANTS,
        notes: "UMI extraction should preserve read counts while enriching read identifiers.",
    },
    StageMetricSpec {
        stage: "fastq.screen_taxonomy",
        classes: &FASTQ_SCREEN_CLASSES,
        invariants: &FASTQ_SCREEN_INVARIANTS,
        notes: "Screening reports contamination only.",
    },
    StageMetricSpec {
        stage: "fastq.deplete_rrna",
        classes: &FASTQ_RRNA_CLASSES,
        invariants: &FASTQ_RRNA_INVARIANTS,
        notes: "rRNA depletion removes classified reads while retaining non-rRNA FASTQ output.",
    },
    StageMetricSpec {
        stage: "fastq.cluster_otus",
        classes: &FASTQ_CLUSTER_OTUS_CLASSES,
        invariants: &FASTQ_CLUSTER_OTUS_INVARIANTS,
        notes: "OTU clustering emits deterministic centroid and abundance artifacts for amplicon workflows.",
    },
    StageMetricSpec {
        stage: "fastq.normalize_primers",
        classes: &FASTQ_NORMALIZE_PRIMERS_CLASSES,
        invariants: &FASTQ_NORMALIZE_PRIMERS_INVARIANTS,
        notes: "Primer normalization re-orients amplicon reads and may trim primer bases while preserving deterministic ordering.",
    },
    StageMetricSpec {
        stage: "fastq.infer_asvs",
        classes: &FASTQ_INFER_ASVS_CLASSES,
        invariants: &FASTQ_INFER_ASVS_INVARIANTS,
        notes: "ASV inference converts denoised amplicon reads into a deterministic feature table and representative sequences.",
    },
    StageMetricSpec {
        stage: "fastq.normalize_abundance",
        classes: &FASTQ_NORMALIZE_ABUNDANCE_CLASSES,
        invariants: &FASTQ_NORMALIZE_ABUNDANCE_INVARIANTS,
        notes: "Abundance normalization operates on a feature table and emits a normalized compositional table.",
    },
    StageMetricSpec {
        stage: "fastq.remove_chimeras",
        classes: &FASTQ_CHIMERA_CLASSES,
        invariants: &FASTQ_CHIMERA_INVARIANTS,
        notes: "Chimera removal filters chimeric amplicon reads while preserving deterministic non-chimera output order.",
    },
    StageMetricSpec {
        stage: "fastq.profile_reads",
        classes: &FASTQ_STATS_CLASSES,
        invariants: &FASTQ_STATS_INVARIANTS,
        notes: "Stats is report-only and must not mutate reads.",
    },
    StageMetricSpec {
        stage: "fastq.profile_read_lengths",
        classes: &FASTQ_READ_LENGTH_CLASSES,
        invariants: &FASTQ_READ_LENGTH_INVARIANTS,
        notes: "Read-length profiling emits deterministic histogram artifacts and summary statistics without mutating reads.",
    },
    StageMetricSpec {
        stage: "fastq.profile_overrepresented_sequences",
        classes: &FASTQ_OVERREPRESENTED_CLASSES,
        invariants: &FASTQ_OVERREPRESENTED_INVARIANTS,
        notes: "Overrepresented sequence profiling is report-only and captures dominant repeated sequence content without mutating reads.",
    },
];

#[must_use]
pub fn metric_spec_for_stage(stage_id: &str) -> Option<StageMetricSpec> {
    FASTQ_STAGE_METRIC_SPECS.iter().find(|spec| spec.stage == stage_id).copied()
}
