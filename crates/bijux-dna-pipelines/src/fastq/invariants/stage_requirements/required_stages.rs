use std::collections::BTreeSet;

use crate::InvariantSeverity;

use super::super::stage_scope::CORE_FASTQ_STAGES;
use super::super::{violation, FastqProfileViolation};
use crate::PipelineProfile;

pub(super) fn push(
    _profile: &PipelineProfile,
    required_stages: &BTreeSet<&str>,
    violations: &mut Vec<FastqProfileViolation>,
) {
    for stage in CORE_FASTQ_STAGES {
        if !required_stages.contains(stage) {
            violations.push(violation(
                "required_stage_missing",
                Some(stage),
                InvariantSeverity::Hard,
                format!("required FASTQ stage `{stage}` is missing"),
            ));
        }
    }
}
