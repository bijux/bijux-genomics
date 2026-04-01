use std::collections::BTreeSet;

use super::FastqProfileViolation;
use crate::PipelineProfile;

mod paired_library_rules;
mod required_artifacts;
mod required_params;
mod required_stages;

pub(super) fn push_required_rule_violations(
    profile: &PipelineProfile,
    required_stages: &BTreeSet<&str>,
    violations: &mut Vec<FastqProfileViolation>,
) {
    required_stages::push(profile, required_stages, violations);
    required_params::push(profile, violations);
    required_artifacts::push(profile, violations);
    paired_library_rules::push(profile, required_stages, violations);
}
