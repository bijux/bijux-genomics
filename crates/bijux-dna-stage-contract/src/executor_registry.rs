//! Code-backed stage executor/readiness registry.

use crate::executor_registry_catalog::ENTRIES;

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

    use crate::executor_registry_catalog::{
        FASTQ_AMPLICON_EXECUTOR, FASTQ_PREPROCESS_EXECUTOR, FASTQ_QC_EXECUTOR,
    };

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
            Some(FASTQ_PREPROCESS_EXECUTOR)
        );
        assert_eq!(
            entry("fastq.report_qc").map(|value| value.executor),
            Some(FASTQ_QC_EXECUTOR)
        );
    }

    #[test]
    fn stage_executor_registry_tracks_supported_infer_asvs_executor() {
        assert!(has_executor("fastq.infer_asvs"));
        assert_eq!(
            entry("fastq.infer_asvs").map(|value| value.executor),
            Some(FASTQ_AMPLICON_EXECUTOR)
        );
    }
}
