pub const DEFAULT_HPC_SUBMISSION_READY_REPORT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/local-ready/HPC_SUBMISSION_READY.json";

pub(super) const DEFAULT_STAGE_COMMANDS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/local-ready/rendered-stage-commands.sh";

pub(super) const DEFAULT_STAGE_OUTPUT_COMPLETION_REPORT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/local-ready/output-completion-report.json";

pub(super) const DEFAULT_STAGE_MANIFEST_COMPLETION_REPORT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/local-ready/manifest-completion-report.json";

pub(super) const DEFAULT_RUNTIME_METRICS_REPORT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/local-ready/runtime-metrics.json";

pub(super) const DEFAULT_TOOL_COMPARISON_TEMPLATE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/local-ready/tool-comparison-template.tsv";

pub(super) const DEFAULT_BENCHMARK_SUMMARY_JSON_PATH: &str =
    "artifacts/benchmark-smoke/readiness/local-ready/benchmark-summary.json";

pub(super) const DEFAULT_BENCHMARK_SUMMARY_MARKDOWN_PATH: &str =
    "artifacts/benchmark-smoke/readiness/local-ready/benchmark-summary.md";

pub(super) const DEFAULT_LOCAL_STAGE_FAILURE_ROOT: &str =
    "artifacts/benchmark-smoke/simulated-runs/failures";

pub(super) const DEFAULT_SLURM_DRY_RUN_ROOT: &str = "artifacts/runs/bench/slurm-dry-run";

pub(super) const DEFAULT_SLURM_SUBMIT_MANIFEST_PATH: &str =
    "artifacts/runs/bench/slurm-dry-run/submit-manifest.json";

pub(super) const DEFAULT_SLURM_DEPENDENCY_CHECK_REPORT_PATH: &str =
    "artifacts/runs/bench/slurm-dry-run/dependency-check.json";

pub(super) const DEFAULT_SLURM_SHELL_SYNTAX_REPORT_PATH: &str =
    "artifacts/runs/bench/slurm-dry-run/bash-n-report.json";

pub(super) const DEFAULT_SLURM_SCRIPT_BODY_REPORT_PATH: &str =
    "artifacts/runs/bench/slurm-dry-run/no-placeholder-report.json";

pub(super) const DEFAULT_CORPUS_SKIP_REPORT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/local-ready/corpus-skip-report.json";

pub(super) const DEFAULT_HPC_SUPPORT_ROOT: &str = "artifacts/hpc/hpc-submission-ready";

pub(super) const FASTQ_STAGE_GOALS: &[(&str, u32, &str)] = &[
    (
        bijux_dna_core::id_catalog::FASTQ_INDEX_REFERENCE,
        2,
        "artifacts/benchmark-smoke/readiness/local-ready/fastq.index_reference",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_VALIDATE_READS,
        3,
        "artifacts/runs/bench/local-smoke/fastq.validate_reads",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_PROFILE_READ_LENGTHS,
        4,
        "artifacts/runs/bench/local-smoke/fastq.profile_read_lengths",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_DETECT_ADAPTERS,
        5,
        "artifacts/runs/bench/local-smoke/fastq.detect_adapters",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_DETECT_DUPLICATES_PREMERGE,
        6,
        "artifacts/runs/bench/local-smoke/fastq.detect_duplicates_premerge",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN,
        7,
        "artifacts/runs/bench/local-smoke/fastq.estimate_library_complexity_prealign",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_TRIM_TERMINAL_DAMAGE,
        8,
        "artifacts/runs/bench/local-smoke/fastq.trim_terminal_damage",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_NORMALIZE_PRIMERS,
        9,
        "artifacts/runs/bench/local-smoke/fastq.normalize_primers",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_TRIM_POLYG_TAILS,
        10,
        "artifacts/runs/bench/local-smoke/fastq.trim_polyg_tails",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_TRIM,
        11,
        "artifacts/runs/bench/local-smoke/fastq.trim_reads",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_FILTER,
        12,
        "artifacts/runs/bench/local-smoke/fastq.filter_reads",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_STATS_NEUTRAL,
        13,
        "artifacts/runs/bench/local-smoke/fastq.profile_reads",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_DEPLETE_RRNA,
        14,
        "artifacts/benchmark-smoke/readiness/local-ready/fastq.deplete_rrna",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_MERGE,
        15,
        "artifacts/runs/bench/local-smoke/fastq.merge_pairs",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_DEDUPLICATE,
        16,
        "artifacts/runs/bench/local-smoke/fastq.remove_duplicates",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_LOW_COMPLEXITY,
        17,
        "artifacts/runs/bench/local-smoke/fastq.filter_low_complexity",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_DEPLETE_HOST,
        18,
        "artifacts/benchmark-smoke/readiness/local-ready/fastq.deplete_host",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_DEPLETE_REFERENCE_CONTAMINANTS,
        19,
        "artifacts/benchmark-smoke/readiness/local-ready/fastq.deplete_reference_contaminants",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_CORRECT,
        20,
        "artifacts/runs/bench/local-smoke/fastq.correct_errors",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_UMI,
        21,
        "artifacts/runs/bench/local-smoke/fastq.extract_umis",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_PROFILE_OVERREPRESENTED_SEQUENCES,
        22,
        "artifacts/runs/bench/local-smoke/fastq.profile_overrepresented_sequences",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_REMOVE_CHIMERAS,
        23,
        "artifacts/runs/bench/local-smoke/fastq.remove_chimeras",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_INFER_ASVS,
        24,
        "artifacts/runs/bench/local-smoke/fastq.infer_asvs",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_CLUSTER_OTUS,
        25,
        "artifacts/runs/bench/local-smoke/fastq.cluster_otus",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_NORMALIZE_ABUNDANCE,
        26,
        "artifacts/runs/bench/local-smoke/fastq.normalize_abundance",
    ),
    (
        bijux_dna_core::id_catalog::FASTQ_SCREEN,
        27,
        "artifacts/benchmark-smoke/readiness/local-ready/fastq.screen_taxonomy",
    ),
];

pub(super) const BAM_STAGE_GOALS: &[(&str, u32, &str)] = &[
    (
        bijux_dna_core::id_catalog::BAM_ALIGN,
        29,
        "artifacts/benchmark-smoke/readiness/local-ready/bam.align",
    ),
    (bijux_dna_core::id_catalog::BAM_VALIDATE, 30, "artifacts/runs/bench/local-smoke/bam.validate"),
    (bijux_dna_core::id_catalog::BAM_QC_PRE, 31, "artifacts/runs/bench/local-smoke/bam.qc_pre"),
    (
        bijux_dna_core::id_catalog::BAM_MAPPING_SUMMARY,
        32,
        "artifacts/runs/bench/local-smoke/bam.mapping_summary",
    ),
    (bijux_dna_core::id_catalog::BAM_FILTER, 33, "artifacts/runs/bench/local-smoke/bam.filter"),
    (
        bijux_dna_core::id_catalog::BAM_MAPQ_FILTER,
        34,
        "artifacts/runs/bench/local-smoke/bam.mapq_filter",
    ),
    (
        bijux_dna_core::id_catalog::BAM_LENGTH_FILTER,
        35,
        "artifacts/runs/bench/local-smoke/bam.length_filter",
    ),
    (bijux_dna_core::id_catalog::BAM_MARKDUP, 36, "artifacts/runs/bench/local-smoke/bam.markdup"),
    (
        bijux_dna_core::id_catalog::BAM_DUPLICATION_METRICS,
        37,
        "artifacts/runs/bench/local-smoke/bam.duplication_metrics",
    ),
    (
        bijux_dna_core::id_catalog::BAM_COMPLEXITY,
        38,
        "artifacts/runs/bench/local-smoke/bam.complexity",
    ),
    (bijux_dna_core::id_catalog::BAM_COVERAGE, 39, "artifacts/runs/bench/local-smoke/bam.coverage"),
    (
        bijux_dna_core::id_catalog::BAM_INSERT_SIZE,
        40,
        "artifacts/runs/bench/local-smoke/bam.insert_size",
    ),
    (bijux_dna_core::id_catalog::BAM_GC_BIAS, 41, "artifacts/runs/bench/local-smoke/bam.gc_bias"),
    (
        bijux_dna_core::id_catalog::BAM_ENDOGENOUS_CONTENT,
        42,
        "artifacts/runs/bench/local-smoke/bam.endogenous_content",
    ),
    (
        bijux_dna_core::id_catalog::BAM_OVERLAP_CORRECTION,
        43,
        "artifacts/runs/bench/local-smoke/bam.overlap_correction",
    ),
    (bijux_dna_core::id_catalog::BAM_DAMAGE, 44, "artifacts/runs/bench/local-smoke/bam.damage"),
    (
        bijux_dna_core::id_catalog::BAM_AUTHENTICITY,
        45,
        "artifacts/runs/bench/local-smoke/bam.authenticity",
    ),
    (
        bijux_dna_core::id_catalog::BAM_CONTAMINATION,
        46,
        "artifacts/benchmark-smoke/readiness/local-ready/bam.contamination",
    ),
    (bijux_dna_core::id_catalog::BAM_SEX, 47, "artifacts/runs/bench/local-smoke/bam.sex"),
    (
        bijux_dna_core::id_catalog::BAM_BIAS_MITIGATION,
        48,
        "artifacts/runs/bench/local-smoke/bam.bias_mitigation",
    ),
    (
        bijux_dna_core::id_catalog::BAM_RECALIBRATION,
        49,
        "artifacts/runs/bench/local-smoke/bam.recalibration",
    ),
    (
        bijux_dna_core::id_catalog::BAM_HAPLOGROUPS,
        50,
        "artifacts/benchmark-smoke/readiness/local-ready/bam.haplogroups",
    ),
    (
        bijux_dna_core::id_catalog::BAM_GENOTYPING,
        51,
        "artifacts/benchmark-smoke/readiness/local-ready/bam.genotyping",
    ),
    (bijux_dna_core::id_catalog::BAM_KINSHIP, 52, "artifacts/runs/bench/local-smoke/bam.kinship"),
];

pub(super) const PIPELINE_DAG_GOALS: &[(u32, &str, &str)] = &[
    (
        77,
        "benchmarks/configs/pipelines/local/fastq-core-preprocess.toml",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/fastq-core-preprocess.json",
    ),
    (
        78,
        "benchmarks/configs/pipelines/local/fastq-paired-merge.toml",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/fastq-paired-merge.json",
    ),
    (
        79,
        "benchmarks/configs/pipelines/local/fastq-edna-taxonomy.toml",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/fastq-edna-taxonomy.json",
    ),
    (
        80,
        "benchmarks/configs/pipelines/local/fastq-amplicon.toml",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/fastq-amplicon.json",
    ),
    (
        81,
        "benchmarks/configs/pipelines/local/fastq-umi.toml",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/fastq-umi.json",
    ),
    (
        82,
        "benchmarks/configs/pipelines/local/bam-core-qc.toml",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/bam-core-qc.json",
    ),
    (
        83,
        "benchmarks/configs/pipelines/local/bam-authenticity.toml",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/bam-authenticity.json",
    ),
    (
        84,
        "benchmarks/configs/pipelines/local/bam-genotyping.toml",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/bam-genotyping.json",
    ),
    (
        85,
        "benchmarks/configs/pipelines/local/bam-kinship.toml",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/bam-kinship.json",
    ),
    (
        86,
        "benchmarks/configs/pipelines/local/fastq-to-bam.toml",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/fastq-to-bam.json",
    ),
];

#[cfg(test)]
mod tests {
    use super::{BAM_STAGE_GOALS, FASTQ_STAGE_GOALS, PIPELINE_DAG_GOALS};
    use std::collections::BTreeSet;

    #[test]
    fn local_stage_goal_mappings_cover_every_numbered_stage_goal_once() {
        let mapped_goal_ids = FASTQ_STAGE_GOALS
            .iter()
            .chain(BAM_STAGE_GOALS.iter())
            .map(|(_, goal_id, _)| *goal_id)
            .collect::<BTreeSet<_>>();

        assert_eq!(FASTQ_STAGE_GOALS.len(), 26);
        assert_eq!(BAM_STAGE_GOALS.len(), 24);
        assert_eq!(mapped_goal_ids.len(), 50);
        assert!(mapped_goal_ids.contains(&2));
        assert!(mapped_goal_ids.contains(&27));
        assert!(mapped_goal_ids.contains(&29));
        assert!(mapped_goal_ids.contains(&52));
    }

    #[test]
    fn pipeline_dag_goal_mappings_cover_every_governed_pipeline_goal_once() {
        let mapped_goal_ids =
            PIPELINE_DAG_GOALS.iter().map(|(goal_id, _, _)| *goal_id).collect::<BTreeSet<_>>();

        assert_eq!(PIPELINE_DAG_GOALS.len(), 10);
        assert_eq!(mapped_goal_ids.len(), 10);
        assert!(mapped_goal_ids.contains(&77));
        assert!(mapped_goal_ids.contains(&86));
    }
}
