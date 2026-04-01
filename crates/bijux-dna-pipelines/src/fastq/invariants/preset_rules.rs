use std::collections::BTreeSet;

use bijux_dna_core::ids::{LibraryLayout, StageId};
use bijux_dna_core::prelude::id_catalog;

use super::{merge_params, screen_params, trim_params, FastqProfileViolation};
use crate::{InvariantSeverity, InvariantsPreset, PipelineProfile};

pub(super) fn push_preset_rule_violations(
    profile: &PipelineProfile,
    required_stages: &BTreeSet<&str>,
    violations: &mut Vec<FastqProfileViolation>,
) {
    let is_adna_like = profile.invariants_preset == Some(InvariantsPreset::Adna)
        || profile.invariants_preset == Some(InvariantsPreset::ReferenceAdna);
    if is_adna_like {
        push_adna_like_rule_violations(profile, required_stages, violations);
    }

    if profile.invariants_preset == Some(InvariantsPreset::ReferenceAdna) {
        push_reference_adna_rule_violations(profile, required_stages, violations);
    }
}

fn push_adna_like_rule_violations(
    profile: &PipelineProfile,
    required_stages: &BTreeSet<&str>,
    violations: &mut Vec<FastqProfileViolation>,
) {
    if let Some(params) = trim_params(profile) {
        if params.min_len == 0 {
            violations.push(super::violation(
                "trim_min_len_invalid",
                Some(id_catalog::FASTQ_TRIM),
                InvariantSeverity::Hard,
                "aDNA profiles must set trim.min_len > 0",
            ));
        }
        if params.adapter_policy.eq_ignore_ascii_case("none") {
            violations.push(super::violation(
                "adna_adapter_policy_invalid",
                Some(id_catalog::FASTQ_TRIM),
                InvariantSeverity::Hard,
                "aDNA profiles must set trim.adapter_policy != \"none\"",
            ));
        }
        if params.q_cutoff.is_none() {
            violations.push(super::violation(
                "adna_quality_trimming_missing",
                Some(id_catalog::FASTQ_TRIM),
                InvariantSeverity::Hard,
                "aDNA profiles must enable quality trimming (trim.q_cutoff)",
            ));
        }
        if params.polyx_policy.is_none() {
            violations.push(super::violation(
                "adna_polyx_trimming_missing",
                Some(id_catalog::FASTQ_TRIM),
                InvariantSeverity::Soft,
                "aDNA profiles must enable poly-X trimming (trim.polyx_policy)",
            ));
        }
    }

    if !required_stages.contains(id_catalog::FASTQ_MERGE) {
        violations.push(super::violation(
            "adna_merge_stage_missing",
            Some(id_catalog::FASTQ_MERGE),
            InvariantSeverity::Hard,
            "aDNA profiles must include fastq.merge_pairs for paired-end collapse/merge",
        ));
    }

    if let Some(params) = merge_params(profile) {
        if params.min_len.unwrap_or(0) == 0 {
            violations.push(super::violation(
                "adna_merge_min_len_invalid",
                Some(id_catalog::FASTQ_MERGE),
                InvariantSeverity::Hard,
                "aDNA profiles must set merge.min_len > 0",
            ));
        }
        if params.merge_overlap.unwrap_or(0) == 0 {
            violations.push(super::violation(
                "adna_merge_overlap_missing",
                Some(id_catalog::FASTQ_MERGE),
                InvariantSeverity::Soft,
                "aDNA profiles should set merge.merge_overlap for aggressive merging",
            ));
        }
    } else {
        violations.push(super::violation(
            "required_params_missing",
            Some(id_catalog::FASTQ_MERGE),
            InvariantSeverity::Hard,
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
        if trim_tool != id_catalog::TOOL_ADAPTERREMOVAL && trim_tool != id_catalog::TOOL_LEEHOM {
            violations.push(super::violation(
                "adna_trim_tool_incompatible",
                Some(id_catalog::FASTQ_TRIM),
                InvariantSeverity::Hard,
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
            violations.push(super::violation(
                "adna_merge_tool_incompatible",
                Some(id_catalog::FASTQ_MERGE),
                InvariantSeverity::Hard,
                "aDNA profiles must use the allowed aDNA merge tool from id_catalog",
            ));
        }
    }
}

fn push_reference_adna_rule_violations(
    profile: &PipelineProfile,
    required_stages: &BTreeSet<&str>,
    violations: &mut Vec<FastqProfileViolation>,
) {
    if profile.library_model.layout != LibraryLayout::PairedEnd {
        violations.push(super::violation(
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
            violations.push(super::violation(
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
            violations.push(super::violation(
                "screen_reference_db_missing",
                Some(id_catalog::FASTQ_SCREEN),
                InvariantSeverity::Soft,
                "fastq.screen_taxonomy requires contaminant_db when enabled for reference-grade profile",
            ));
        }
    }
}
