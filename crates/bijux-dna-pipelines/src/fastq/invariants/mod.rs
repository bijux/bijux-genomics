//! FASTQ pipeline profile invariants and validation.

use crate::PipelineProfile;

mod preset_rules;
mod stage_parameter_access;
mod stage_scope;
mod stage_requirements;
mod validation_report_contracts;
mod violation_builder;

use preset_rules::push_preset_rule_violations;
use stage_scope::stage_set;
use stage_requirements::push_required_rule_violations;
pub use validation_report_contracts::{
    FastqProfileValidationReport, FastqProfileViolation, FASTQ_INVARIANTS,
};
use violation_builder::violation;
use stage_parameter_access::{
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
