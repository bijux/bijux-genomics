//! FASTQ pipeline profile invariants and validation.

use std::collections::BTreeSet;

use bijux_dna_core::ids::StageId;
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_fastq::params::detect_adapters::DetectAdaptersEffectiveParams;
use bijux_dna_domain_fastq::params::filter::FilterEffectiveParams;
use bijux_dna_domain_fastq::params::merge::MergeEffectiveParams;
use bijux_dna_domain_fastq::params::preprocess::PreprocessEffectiveParams;
use bijux_dna_domain_fastq::params::screen::ScreenEffectiveParams;
use bijux_dna_domain_fastq::params::trim::TrimEffectiveParams;
use serde::Serialize;

use crate::{DefaultParams, InvariantsPreset, PipelineProfile};

pub const FASTQ_INVARIANTS: &str = "fastq-invariants.v2";

const CORE_FASTQ_STAGES: [&str; 5] = [
    id_catalog::FASTQ_VALIDATE_PRE,
    id_catalog::FASTQ_DETECT_ADAPTERS,
    id_catalog::FASTQ_TRIM,
    id_catalog::FASTQ_FILTER,
    id_catalog::FASTQ_QC_POST,
];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FastqProfileViolation {
    pub code: &'static str,
    pub stage_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FastqProfileValidationReport {
    pub profile_id: String,
    pub invariants_version: &'static str,
    pub invariants_preset: Option<String>,
    pub valid: bool,
    pub violations: Vec<FastqProfileViolation>,
}

impl FastqProfileValidationReport {
    #[must_use]
    pub fn from_violations(
        profile: &PipelineProfile,
        violations: Vec<FastqProfileViolation>,
    ) -> Self {
        Self {
            profile_id: profile.id.as_str().to_string(),
            invariants_version: FASTQ_INVARIANTS,
            invariants_preset: profile
                .invariants_preset
                .map(|preset| preset.as_str().to_string()),
            valid: violations.is_empty(),
            violations,
        }
    }
}

fn violation(
    code: &'static str,
    stage_id: Option<&str>,
    message: impl Into<String>,
) -> FastqProfileViolation {
    FastqProfileViolation {
        code,
        stage_id: stage_id.map(str::to_string),
        message: message.into(),
    }
}

fn stage_set(profile: &PipelineProfile) -> BTreeSet<&str> {
    profile
        .capabilities
        .required_stages
        .iter()
        .copied()
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

fn preprocess_params(profile: &PipelineProfile) -> Option<&PreprocessEffectiveParams> {
    match default_params_for(profile, id_catalog::FASTQ_PREPROCESS) {
        Some(DefaultParams::FastqPreprocess(params)) => Some(params),
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

    for stage in CORE_FASTQ_STAGES {
        if !required_stages.contains(stage) {
            violations.push(violation(
                "required_stage_missing",
                Some(stage),
                format!("required FASTQ stage `{stage}` is missing"),
            ));
        }
    }

    if trim_params(profile).is_none() {
        violations.push(violation(
            "required_params_missing",
            Some(id_catalog::FASTQ_TRIM),
            "missing or invalid trim params",
        ));
    }
    if filter_params(profile).is_none() {
        violations.push(violation(
            "required_params_missing",
            Some(id_catalog::FASTQ_FILTER),
            "missing or invalid filter params",
        ));
    }
    if detect_adapters_params(profile).is_none() {
        violations.push(violation(
            "required_params_missing",
            Some(id_catalog::FASTQ_DETECT_ADAPTERS),
            "missing or invalid detect_adapters params",
        ));
    }

    if let Some(params) = trim_params(profile) {
        if params.adapter_policy.trim().is_empty() {
            violations.push(violation(
                "trim_adapter_policy_invalid",
                Some(id_catalog::FASTQ_TRIM),
                "trim.adapter_policy must be non-empty",
            ));
        }
    }

    let is_adna_like = profile.invariants_preset == Some(InvariantsPreset::Adna)
        || profile.invariants_preset == Some(InvariantsPreset::ReferenceAdna);
    if is_adna_like {
        if let Some(params) = trim_params(profile) {
            if params.min_len == 0 {
                violations.push(violation(
                    "trim_min_len_invalid",
                    Some(id_catalog::FASTQ_TRIM),
                    "aDNA profiles must set trim.min_len > 0",
                ));
            }
            if params.adapter_policy.eq_ignore_ascii_case("none") {
                violations.push(violation(
                    "adna_adapter_policy_invalid",
                    Some(id_catalog::FASTQ_TRIM),
                    "aDNA profiles must set trim.adapter_policy != \"none\"",
                ));
            }
            if params.q_cutoff.is_none() {
                violations.push(violation(
                    "adna_quality_trimming_missing",
                    Some(id_catalog::FASTQ_TRIM),
                    "aDNA profiles must enable quality trimming (trim.q_cutoff)",
                ));
            }
            if params.polyx_policy.is_none() {
                violations.push(violation(
                    "adna_polyx_trimming_missing",
                    Some(id_catalog::FASTQ_TRIM),
                    "aDNA profiles must enable poly-X trimming (trim.polyx_policy)",
                ));
            }
        }

        if !required_stages.contains(id_catalog::FASTQ_MERGE) {
            violations.push(violation(
                "adna_merge_stage_missing",
                Some(id_catalog::FASTQ_MERGE),
                "aDNA profiles must include fastq.merge for paired-end collapse/merge",
            ));
        }

        if let Some(params) = merge_params(profile) {
            if params.min_len.unwrap_or(0) == 0 {
                violations.push(violation(
                    "adna_merge_min_len_invalid",
                    Some(id_catalog::FASTQ_MERGE),
                    "aDNA profiles must set merge.min_len > 0",
                ));
            }
            if params.merge_overlap.unwrap_or(0) == 0 {
                violations.push(violation(
                    "adna_merge_overlap_missing",
                    Some(id_catalog::FASTQ_MERGE),
                    "aDNA profiles should set merge.merge_overlap for aggressive merging",
                ));
            }
        } else {
            violations.push(violation(
                "required_params_missing",
                Some(id_catalog::FASTQ_MERGE),
                "missing or invalid merge params",
            ));
        }

        if profile.invariants_preset == Some(InvariantsPreset::Adna) {
            let trim_tool = profile
                .defaults
                .tools
                .get(&StageId::from_static(id_catalog::FASTQ_TRIM))
                .map(|tool| tool.as_str())
                .unwrap_or_default();
            if trim_tool != id_catalog::TOOL_ADAPTERREMOVAL && trim_tool != id_catalog::TOOL_LEEHOM
            {
                violations.push(violation(
                    "adna_trim_tool_incompatible",
                    Some(id_catalog::FASTQ_TRIM),
                    "aDNA profiles must use an allowed aDNA trim tool from id_catalog",
                ));
            }

            let merge_tool = profile
                .defaults
                .tools
                .get(&StageId::from_static(id_catalog::FASTQ_MERGE))
                .map(|tool| tool.as_str())
                .unwrap_or_default();
            if merge_tool != id_catalog::TOOL_LEEHOM {
                violations.push(violation(
                    "adna_merge_tool_incompatible",
                    Some(id_catalog::FASTQ_MERGE),
                    "aDNA profiles must use the allowed aDNA merge tool from id_catalog",
                ));
            }
        }
    }

    if profile.invariants_preset == Some(InvariantsPreset::ReferenceAdna) {
        for stage in [
            id_catalog::FASTQ_LOW_COMPLEXITY,
            id_catalog::FASTQ_STATS_NEUTRAL,
            id_catalog::FASTQ_MERGE,
        ] {
            if !required_stages.contains(stage) {
                violations.push(violation(
                    "reference_required_stage_missing",
                    Some(stage),
                    format!("reference-grade aDNA profile requires stage `{stage}`"),
                ));
            }
        }

        if let Some(preprocess) = preprocess_params(profile) {
            if preprocess.library_declared_paired
                && !required_stages.contains(id_catalog::FASTQ_MERGE)
            {
                violations.push(violation(
                    "paired_library_requires_merge",
                    Some(id_catalog::FASTQ_MERGE),
                    "paired library declaration requires fastq.merge unless explicitly disabled with justification",
                ));
            }
        } else {
            violations.push(violation(
                "required_params_missing",
                Some(id_catalog::FASTQ_PREPROCESS),
                "reference-grade profile requires preprocess library declaration params",
            ));
        }

        if required_stages.contains(id_catalog::FASTQ_SCREEN) {
            let missing_db = screen_params(profile)
                .and_then(|params| params.contaminant_db.as_ref())
                .map_or(true, |value| value.trim().is_empty());
            if missing_db {
                violations.push(violation(
                    "screen_reference_db_missing",
                    Some(id_catalog::FASTQ_SCREEN),
                    "fastq.screen requires contaminant_db when enabled for reference-grade profile",
                ));
            }
        }
    }

    FastqProfileValidationReport::from_violations(profile, violations)
}
