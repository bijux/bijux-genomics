//! FASTQ pipeline profile invariants and validation.

use std::collections::BTreeSet;

use bijux_dna_core::ids::StageId;
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_fastq::params::detect_adapters::DetectAdaptersEffectiveParams;
use bijux_dna_domain_fastq::params::filter::FilterEffectiveParams;
use bijux_dna_domain_fastq::params::merge::MergeEffectiveParams;
use bijux_dna_domain_fastq::params::screen::ScreenEffectiveParams;
use bijux_dna_domain_fastq::params::trim::TrimEffectiveParams;

use crate::{DefaultParams, PipelineProfile};

mod preset_rules;
mod report;
mod required_rules;

pub use report::{FastqProfileValidationReport, FastqProfileViolation, FASTQ_INVARIANTS};
use preset_rules::push_preset_rule_violations;
use report::violation;
use required_rules::push_required_rule_violations;

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

fn default_params_for<'a>(
    profile: &'a PipelineProfile,
    stage_id: &str,
) -> Option<&'a DefaultParams> {
    let stage = StageId::new(stage_id.to_string());
    profile.defaults.params.get(&stage)
}

fn trim_params(profile: &PipelineProfile) -> Option<&TrimEffectiveParams> {
    match default_params_for(profile, id_catalog::FASTQ_TRIM) {
        Some(DefaultParams::FastqTrim(params)) => Some(params),
        _ => None,
    }
}

fn filter_params(profile: &PipelineProfile) -> Option<&FilterEffectiveParams> {
    match default_params_for(profile, id_catalog::FASTQ_FILTER) {
        Some(DefaultParams::FastqFilter(params)) => Some(params),
        _ => None,
    }
}

fn detect_adapters_params(profile: &PipelineProfile) -> Option<&DetectAdaptersEffectiveParams> {
    match default_params_for(profile, id_catalog::FASTQ_DETECT_ADAPTERS) {
        Some(DefaultParams::FastqDetectAdapters(params)) => Some(params),
        _ => None,
    }
}

fn merge_params(profile: &PipelineProfile) -> Option<&MergeEffectiveParams> {
    match default_params_for(profile, id_catalog::FASTQ_MERGE) {
        Some(DefaultParams::FastqMerge(params)) => Some(params),
        _ => None,
    }
}

fn screen_params(profile: &PipelineProfile) -> Option<&ScreenEffectiveParams> {
    match default_params_for(profile, id_catalog::FASTQ_SCREEN) {
        Some(DefaultParams::FastqScreen(params)) => Some(params),
        _ => None,
    }
}

/// Validate FASTQ profile invariants and return a structured violations report.
#[must_use]
pub fn validate_fastq_profile(profile: &PipelineProfile) -> FastqProfileValidationReport {
    let mut violations = Vec::new();
    let required_stages = stage_set(profile);

    push_required_rule_violations(profile, &required_stages, &mut violations);
    push_preset_rule_violations(profile, &required_stages, &mut violations);

    FastqProfileValidationReport::from_violations(profile, violations)
}
