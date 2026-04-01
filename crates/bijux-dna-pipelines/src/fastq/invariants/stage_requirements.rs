use std::collections::BTreeSet;

use bijux_dna_core::ids::LibraryLayout;
use bijux_dna_core::prelude::id_catalog;

use super::{
    detect_adapters_params, filter_params, trim_params, FastqProfileViolation, CORE_FASTQ_STAGES,
};
use crate::{InvariantSeverity, PipelineProfile};

pub(super) fn push_required_rule_violations(
    profile: &PipelineProfile,
    required_stages: &BTreeSet<&str>,
    violations: &mut Vec<FastqProfileViolation>,
) {
    for stage in CORE_FASTQ_STAGES {
        if !required_stages.contains(stage) {
            violations.push(super::violation(
                "required_stage_missing",
                Some(stage),
                InvariantSeverity::Hard,
                format!("required FASTQ stage `{stage}` is missing"),
            ));
        }
    }

    if trim_params(profile).is_none() {
        violations.push(super::violation(
            "required_params_missing",
            Some(id_catalog::FASTQ_TRIM),
            InvariantSeverity::Hard,
            "missing or invalid trim params",
        ));
    }
    if filter_params(profile).is_none() {
        violations.push(super::violation(
            "required_params_missing",
            Some(id_catalog::FASTQ_FILTER),
            InvariantSeverity::Hard,
            "missing or invalid filter params",
        ));
    }
    if detect_adapters_params(profile).is_none() {
        violations.push(super::violation(
            "required_params_missing",
            Some(id_catalog::FASTQ_DETECT_ADAPTERS),
            InvariantSeverity::Hard,
            "missing or invalid detect_adapters params",
        ));
    }

    if let Some(params) = trim_params(profile) {
        if params.adapter_policy.trim().is_empty() {
            violations.push(super::violation(
                "trim_adapter_policy_invalid",
                Some(id_catalog::FASTQ_TRIM),
                InvariantSeverity::Hard,
                "trim.adapter_policy must be non-empty",
            ));
        }
    }

    for artifact in [
        "report.json",
        "run_manifest.json",
        "stage_summaries.json",
        "invariants_report.json",
    ] {
        if !profile.capabilities.required_artifacts.contains(&artifact) {
            violations.push(super::violation(
                "required_artifact_missing",
                None,
                InvariantSeverity::Hard,
                format!("FASTQ profiles must require `{artifact}`"),
            ));
        }
    }

    if profile.library_model.layout == LibraryLayout::PairedEnd
        && !required_stages.contains(id_catalog::FASTQ_MERGE)
    {
        violations.push(super::violation(
            "paired_library_requires_merge",
            Some(id_catalog::FASTQ_MERGE),
            InvariantSeverity::Hard,
            "paired library declaration requires fastq.merge_pairs unless explicitly disabled with justification",
        ));
    }
}
