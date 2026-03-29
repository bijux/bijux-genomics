//! Code-backed executor registry catalog entries.

use crate::executor_registry::{ReadinessBadge, StageDomain, StageExecutorEntry};

pub(crate) const FASTQ_PREPROCESS_EXECUTOR: &str = "api.fastq.preprocess";
pub(crate) const FASTQ_QC_EXECUTOR: &str = "api.fastq.qc";
pub(crate) const FASTQ_REFERENCE_EXECUTOR: &str = "api.fastq.reference";
pub(crate) const FASTQ_AMPLICON_EXECUTOR: &str = "api.fastq.amplicon";
pub(crate) const FASTQ_CLASSIFY_EXECUTOR: &str = "api.fastq.classify";
pub(crate) const BAM_EXECUTOR: &str = "api.bam.exec";
pub(crate) const VCF_EXECUTOR: &str = "stages-vcf.pipeline";

pub(crate) const ENTRIES: &[StageExecutorEntry] = &[
    StageExecutorEntry {
        stage_id: "fastq.normalize_abundance",
        executor: FASTQ_AMPLICON_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.infer_asvs",
        executor: FASTQ_AMPLICON_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.remove_chimeras",
        executor: FASTQ_AMPLICON_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.deplete_reference_contaminants",
        executor: FASTQ_REFERENCE_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.correct_errors",
        executor: FASTQ_PREPROCESS_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.trim_terminal_damage",
        executor: FASTQ_AMPLICON_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.remove_duplicates",
        executor: FASTQ_PREPROCESS_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.detect_adapters",
        executor: FASTQ_PREPROCESS_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.filter_reads",
        executor: FASTQ_PREPROCESS_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.deplete_host",
        executor: FASTQ_PREPROCESS_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.profile_read_lengths",
        executor: FASTQ_QC_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.filter_low_complexity",
        executor: FASTQ_PREPROCESS_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.merge_pairs",
        executor: FASTQ_PREPROCESS_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.cluster_otus",
        executor: FASTQ_AMPLICON_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.profile_overrepresented_sequences",
        executor: FASTQ_QC_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.trim_polyg_tails",
        executor: FASTQ_PREPROCESS_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.index_reference",
        executor: FASTQ_AMPLICON_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.normalize_primers",
        executor: FASTQ_QC_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.report_qc",
        executor: FASTQ_QC_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.deplete_rrna",
        executor: FASTQ_QC_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.screen_taxonomy",
        executor: FASTQ_CLASSIFY_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.profile_reads",
        executor: FASTQ_QC_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.trim_reads",
        executor: FASTQ_PREPROCESS_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.extract_umis",
        executor: FASTQ_PREPROCESS_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.validate_reads",
        executor: FASTQ_QC_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.align",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.bias_mitigation",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.complexity",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.authenticity",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.contamination",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.coverage",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.damage",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.duplication_metrics",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.endogenous_content",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.filter",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.gc_bias",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.genotyping",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.haplogroups",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.insert_size",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.kinship",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.length_filter",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.markdup",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.mapping_summary",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.mapq_filter",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.overlap_correction",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.qc_pre",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.recalibration",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.sex",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "bam.validate",
        executor: BAM_EXECUTOR,
        domain: StageDomain::Bam,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "vcf.call",
        executor: VCF_EXECUTOR,
        domain: StageDomain::Vcf,
        readiness: ReadinessBadge::Experimental,
    },
    StageExecutorEntry {
        stage_id: "vcf.call_diploid",
        executor: VCF_EXECUTOR,
        domain: StageDomain::Vcf,
        readiness: ReadinessBadge::Experimental,
    },
    StageExecutorEntry {
        stage_id: "vcf.call_gl",
        executor: VCF_EXECUTOR,
        domain: StageDomain::Vcf,
        readiness: ReadinessBadge::Experimental,
    },
    StageExecutorEntry {
        stage_id: "vcf.call_pseudohaploid",
        executor: VCF_EXECUTOR,
        domain: StageDomain::Vcf,
        readiness: ReadinessBadge::Experimental,
    },
    StageExecutorEntry {
        stage_id: "vcf.damage_filter",
        executor: VCF_EXECUTOR,
        domain: StageDomain::Vcf,
        readiness: ReadinessBadge::Experimental,
    },
    StageExecutorEntry {
        stage_id: "vcf.filter",
        executor: VCF_EXECUTOR,
        domain: StageDomain::Vcf,
        readiness: ReadinessBadge::Experimental,
    },
    StageExecutorEntry {
        stage_id: "vcf.gl_propagation",
        executor: VCF_EXECUTOR,
        domain: StageDomain::Vcf,
        readiness: ReadinessBadge::Experimental,
    },
    StageExecutorEntry {
        stage_id: "vcf.stats",
        executor: VCF_EXECUTOR,
        domain: StageDomain::Vcf,
        readiness: ReadinessBadge::Experimental,
    },
];
