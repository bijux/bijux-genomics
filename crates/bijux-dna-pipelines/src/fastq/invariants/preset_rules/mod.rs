use std::collections::BTreeSet;

use super::FastqProfileViolation;
use crate::{InvariantSeverity, InvariantsPreset, PipelineProfile};

mod ancient_dna_rules;
mod reference_adna_rules;

pub(super) fn push_preset_rule_violations(
    profile: &PipelineProfile,
    required_stages: &BTreeSet<&str>,
    violations: &mut Vec<FastqProfileViolation>,
) {
    let is_adna_like = profile.invariants_preset == Some(InvariantsPreset::Adna)
        || profile.invariants_preset == Some(InvariantsPreset::ReferenceAdna);
    if is_adna_like {
        ancient_dna_rules::push_ancient_dna_rule_violations(profile, required_stages, violations);
    }

    if profile.invariants_preset == Some(InvariantsPreset::ReferenceAdna) {
        reference_adna_rules::push_reference_adna_rule_violations(
            profile,
            required_stages,
            violations,
        );
    }
}
