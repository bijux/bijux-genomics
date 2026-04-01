use bijux_dna_core::prelude::id_catalog;

use crate::InvariantSeverity;

use super::super::{
    detect_adapters_params, filter_params, trim_params, violation, FastqProfileViolation,
};
use crate::PipelineProfile;

pub(super) fn push(profile: &PipelineProfile, violations: &mut Vec<FastqProfileViolation>) {
    if trim_params(profile).is_none() {
        violations.push(violation(
            "required_params_missing",
            Some(id_catalog::FASTQ_TRIM),
            InvariantSeverity::Hard,
            "missing or invalid trim params",
        ));
    }
    if filter_params(profile).is_none() {
        violations.push(violation(
            "required_params_missing",
            Some(id_catalog::FASTQ_FILTER),
            InvariantSeverity::Hard,
            "missing or invalid filter params",
        ));
    }
    if detect_adapters_params(profile).is_none() {
        violations.push(violation(
            "required_params_missing",
            Some(id_catalog::FASTQ_DETECT_ADAPTERS),
            InvariantSeverity::Hard,
            "missing or invalid detect_adapters params",
        ));
    }

    if let Some(params) = trim_params(profile) {
        if params.adapter_policy.trim().is_empty() {
            violations.push(violation(
                "trim_adapter_policy_invalid",
                Some(id_catalog::FASTQ_TRIM),
                InvariantSeverity::Hard,
                "trim.adapter_policy must be non-empty",
            ));
        }
    }
}
