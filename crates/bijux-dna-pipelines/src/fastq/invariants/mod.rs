//! FASTQ pipeline profile invariants and validation.

use std::collections::BTreeSet;

use crate::PipelineProfile;
use bijux_dna_core::prelude::id_catalog;

mod preset_rules;
mod required_rules;
mod stage_params;
mod validation_report;

use preset_rules::push_preset_rule_violations;
use required_rules::push_required_rule_violations;
use validation_report::violation;
pub use validation_report::{
    FastqProfileValidationReport, FastqProfileViolation, FASTQ_INVARIANTS,
};

const CORE_FASTQ_STAGES: [&str; 5] = [
    id_catalog::FASTQ_VALIDATE_PRE,
    id_catalog::FASTQ_DETECT_ADAPTERS,
    id_catalog::FASTQ_TRIM,
    id_catalog::FASTQ_FILTER,
    id_catalog::FASTQ_QC_POST,
];

fn stage_set(profile: &PipelineProfile) -> BTreeSet<&str> {
    profile
        .capabilities
        .required_stages
        .iter()
        .map(String::as_str)
        .collect()
}
use stage_params::{
    detect_adapters_params, filter_params, merge_params, screen_params, trim_params,
};

/// Validate FASTQ profile invariants and return a structured violations report.
#[must_use]
pub fn validate_fastq_profile(profile: &PipelineProfile) -> FastqProfileValidationReport {
    let mut violations = Vec::new();
    let required_stages = stage_set(profile);

    push_required_rule_violations(profile, &required_stages, &mut violations);
    push_preset_rule_violations(profile, &required_stages, &mut violations);

    FastqProfileValidationReport::from_violations(profile, violations)
}
