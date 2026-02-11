//! FASTQ pipeline profile invariants and validation.

use std::collections::BTreeSet;

use bijux_dna_core::ids::StageId;
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_fastq::params::detect_adapters::DetectAdaptersEffectiveParams;
use bijux_dna_domain_fastq::params::filter::FilterEffectiveParams;
use bijux_dna_domain_fastq::params::merge::MergeEffectiveParams;
use bijux_dna_domain_fastq::params::trim::TrimEffectiveParams;
use serde::Serialize;

use crate::{DefaultParams, PipelineProfile};

pub const FASTQ_INVARIANTS: &str = "fastq-invariants.v1";

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
            invariants_preset: profile.invariants_preset.map(str::to_string),
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
    profile.defaults.params.get(&StageId::from_static(stage_id))
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

    let is_adna = profile.invariants_preset == Some("adna");
    if is_adna {
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

        let trim_tool = profile
            .defaults
            .tools
            .get(&StageId::from_static(id_catalog::FASTQ_TRIM))
            .map(|tool| tool.as_str())
            .unwrap_or_default();
        if trim_tool != "adapterremoval" && trim_tool != "leehom" {
            violations.push(violation(
                "adna_trim_tool_incompatible",
                Some(id_catalog::FASTQ_TRIM),
                "aDNA profiles must use trim tool `adapterremoval` or `leehom`",
            ));
        }

        let merge_tool = profile
            .defaults
            .tools
            .get(&StageId::from_static(id_catalog::FASTQ_MERGE))
            .map(|tool| tool.as_str())
            .unwrap_or_default();
        if merge_tool != "leehom" {
            violations.push(violation(
                "adna_merge_tool_incompatible",
                Some(id_catalog::FASTQ_MERGE),
                "aDNA profiles must use merge tool `leehom`",
            ));
        }
    }

    FastqProfileValidationReport::from_violations(profile, violations)
}
