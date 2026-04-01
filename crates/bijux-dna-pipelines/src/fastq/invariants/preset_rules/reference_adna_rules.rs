use std::collections::BTreeSet;

use bijux_dna_core::ids::LibraryLayout;
use bijux_dna_core::prelude::id_catalog;

use crate::fastq::invariants::{screen_params, FastqProfileViolation};
use crate::{InvariantSeverity, PipelineProfile};

pub(super) fn push_reference_adna_rule_violations(
    profile: &PipelineProfile,
    required_stages: &BTreeSet<&str>,
    violations: &mut Vec<FastqProfileViolation>,
) {
    if profile.library_model.layout != LibraryLayout::PairedEnd {
        violations.push(super::super::violation(
            "reference_library_layout_invalid",
            None,
            InvariantSeverity::Hard,
            "reference-grade aDNA profile must declare paired-end library layout",
        ));
    }

    for stage in [
        id_catalog::FASTQ_LOW_COMPLEXITY,
        id_catalog::FASTQ_STATS_NEUTRAL,
        id_catalog::FASTQ_MERGE,
    ] {
        if !required_stages.contains(stage) {
            violations.push(super::super::violation(
                "reference_required_stage_missing",
                Some(stage),
                InvariantSeverity::Hard,
                format!("reference-grade aDNA profile requires stage `{stage}`"),
            ));
        }
    }

    if required_stages.contains(id_catalog::FASTQ_SCREEN) {
        let missing_db = screen_params(profile)
            .and_then(|params| params.contaminant_db.as_ref())
            .map_or(true, |value| value.trim().is_empty());
        if missing_db {
            violations.push(super::super::violation(
                "screen_reference_db_missing",
                Some(id_catalog::FASTQ_SCREEN),
                InvariantSeverity::Soft,
                "fastq.screen_taxonomy requires contaminant_db when enabled for reference-grade profile",
            ));
        }
    }
}
