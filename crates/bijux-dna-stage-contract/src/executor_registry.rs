//! Code-backed stage executor/readiness registry.

/// CI/governance readiness badge for a stage executor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadinessBadge {
    /// Implemented but not admitted to stable profiles.
    Experimental,
    /// Implemented and admitted to stable profiles.
    Supported,
    /// Implemented, validated, and locked for high-assurance profiles.
    Certified,
}

/// Domain label for stage ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageDomain {
    /// FASTQ domain.
    Fastq,
    /// BAM domain.
    Bam,
    /// VCF domain.
    Vcf,
}

/// Code-backed executor entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageExecutorEntry {
    /// Stage identifier (`domain.verb`).
    pub stage_id: &'static str,
    /// Canonical executor binding label.
    pub executor: &'static str,
    /// Owning domain.
    pub domain: StageDomain,
    /// Readiness badge.
    pub readiness: ReadinessBadge,
}

const FASTQ_EXECUTOR: &str = "api.fastq.preprocess";
const BAM_EXECUTOR: &str = "api.bam.exec";
const VCF_EXECUTOR: &str = "stages-vcf.pipeline";

const ENTRIES: &[StageExecutorEntry] = &[
    StageExecutorEntry {
        stage_id: "fastq.abundance_normalization",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.chimera_detection",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.deplete_reference_contaminants",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.correct_errors",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.trim_terminal_damage",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.remove_duplicates",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.detect_adapters",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.filter_reads",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.deplete_host",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.profile_read_lengths",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.filter_low_complexity",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.merge_pairs",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.otu_clustering",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.profile_overrepresented_sequences",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.trim_polyg_tails",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.prepare_reference",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.primer_normalization",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.report_qc",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.deplete_rrna",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.screen_taxonomy",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.profile_reads",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.trim_reads",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.extract_umis",
        executor: FASTQ_EXECUTOR,
        domain: StageDomain::Fastq,
        readiness: ReadinessBadge::Supported,
    },
    StageExecutorEntry {
        stage_id: "fastq.validate_reads",
        executor: FASTQ_EXECUTOR,
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

/// Returns all code-backed executor entries.
#[must_use]
pub fn entries() -> &'static [StageExecutorEntry] {
    ENTRIES
}

/// Returns `true` when a stage has a code-backed executor.
#[must_use]
pub fn has_executor(stage_id: &str) -> bool {
    ENTRIES.iter().any(|entry| entry.stage_id == stage_id)
}

/// Returns the executor entry for a stage id.
#[must_use]
pub fn entry(stage_id: &str) -> Option<StageExecutorEntry> {
    ENTRIES.iter().copied().find(|e| e.stage_id == stage_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn stage_executor_registry_has_unique_stage_ids() {
        let ids = entries()
            .iter()
            .map(|entry| entry.stage_id)
            .collect::<Vec<_>>();
        let uniq = ids.iter().copied().collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), uniq.len(), "duplicate stage ids in registry");
    }

    #[test]
    fn stage_executor_registry_surfaces_lookup_helpers() {
        let _ = ReadinessBadge::Certified;

        assert!(has_executor("fastq.filter_reads"));
        assert_eq!(
            entry("fastq.filter_reads").map(|value| value.executor),
            Some(FASTQ_EXECUTOR)
        );
    }
}
