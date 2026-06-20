//! Executor registry lookup surface and invariants.

use crate::executor_registry::catalog::ENTRIES;
use crate::executor_registry::StageExecutorEntry;

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
    ENTRIES.iter().copied().find(|candidate| candidate.stage_id == stage_id)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use bijux_dna_core::id_catalog;

    use crate::executor_registry::catalog::{
        FASTQ_AMPLICON_EXECUTOR, FASTQ_PREPROCESS_EXECUTOR, FASTQ_QC_EXECUTOR,
    };
    use crate::executor_registry::ReadinessBadge;

    use super::{entries, entry, has_executor};

    #[test]
    fn stage_executor_registry_has_unique_stage_ids() {
        let ids = entries().iter().map(|entry| entry.stage_id).collect::<Vec<_>>();
        let uniq = ids.iter().copied().collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), uniq.len(), "duplicate stage ids in registry");
    }

    #[test]
    fn stage_executor_registry_surfaces_lookup_helpers() {
        let _ = ReadinessBadge::Certified;

        assert!(has_executor(id_catalog::FASTQ_FILTER));
        assert_eq!(
            entry(id_catalog::FASTQ_FILTER).map(|value| value.executor),
            Some(FASTQ_PREPROCESS_EXECUTOR)
        );
        assert_eq!(
            entry(id_catalog::FASTQ_QC_POST).map(|value| value.executor),
            Some(FASTQ_QC_EXECUTOR)
        );
    }

    #[test]
    fn stage_executor_registry_tracks_supported_infer_asvs_executor() {
        assert!(has_executor(id_catalog::FASTQ_INFER_ASVS));
        assert_eq!(
            entry(id_catalog::FASTQ_INFER_ASVS).map(|value| value.executor),
            Some(FASTQ_AMPLICON_EXECUTOR)
        );
    }

    #[test]
    fn stage_executor_registry_tracks_supported_vcf_population_executors() {
        assert!(has_executor(id_catalog::VCF_ADMIXTURE));
        assert!(has_executor(id_catalog::VCF_ROH));
    }
}
