use std::collections::BTreeSet;

use bijux_dna_core::ids::LibraryLayout;
use bijux_dna_core::prelude::id_catalog;

use crate::InvariantSeverity;

use super::super::{violation, FastqProfileViolation};
use crate::PipelineProfile;

pub(super) fn push(
    profile: &PipelineProfile,
    required_stages: &BTreeSet<&str>,
    violations: &mut Vec<FastqProfileViolation>,
) {
    if profile.library_model.layout == LibraryLayout::PairedEnd
        && !required_stages.contains(id_catalog::FASTQ_MERGE)
    {
        violations.push(violation(
            "paired_library_requires_merge",
            Some(id_catalog::FASTQ_MERGE),
            InvariantSeverity::Hard,
            "paired library declaration requires fastq.merge_pairs unless explicitly disabled with justification",
        ));
    }
}
