//! VCF profile invariants and validation.

use std::collections::BTreeSet;

use bijux_dna_core::ids::StageId;
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_vcf::params::VcfEffectiveParams;
use serde::Serialize;

use crate::{
    DefaultParams, InvariantSeverity, InvariantViolationV1, InvariantsReportV1, PipelineProfile,
    StabilityTier,
};

pub const VCF_INVARIANTS: &str = "vcf-invariants.v1";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VcfProfileViolation {
    pub code: &'static str,
    pub stage_id: Option<String>,
    pub severity: InvariantSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VcfProfileValidationReport {
    pub profile_id: String,
    pub invariants_version: &'static str,
    pub valid: bool,
    pub violations: Vec<VcfProfileViolation>,
}

impl VcfProfileValidationReport {
    #[must_use]
    pub fn as_invariants_report(&self) -> InvariantsReportV1 {
        InvariantsReportV1 {
            schema_version: "bijux.invariants_report.v1".to_string(),
            profile_id: self.profile_id.clone(),
            invariants_version: self.invariants_version.to_string(),
            valid: self.valid,
            blocking: self
                .violations
                .iter()
                .any(|v| v.severity == InvariantSeverity::Hard),
            violations: self
                .violations
                .iter()
                .map(|v| InvariantViolationV1 {
                    code: v.code.to_string(),
                    stage_id: v.stage_id.clone(),
                    severity: v.severity,
                    message: v.message.clone(),
                })
                .collect(),
        }
    }
}

fn violation(
    code: &'static str,
    stage_id: Option<&str>,
    severity: InvariantSeverity,
    message: impl Into<String>,
) -> VcfProfileViolation {
    VcfProfileViolation {
        code,
        stage_id: stage_id.map(str::to_string),
        severity,
        message: message.into(),
    }
}

fn stage_set(profile: &PipelineProfile) -> BTreeSet<&str> {
    profile
        .capabilities
        .required_stages
        .iter()
        .map(String::as_str)
        .collect()
}

#[must_use]
pub fn validate_vcf_profile(profile: &PipelineProfile) -> VcfProfileValidationReport {
    let mut violations = Vec::new();
    let stages = stage_set(profile);

    for stage in [
        id_catalog::VCF_CALL,
        id_catalog::VCF_FILTER,
        id_catalog::VCF_STATS,
    ] {
        let stage_id = StageId::from_static(stage);
        if !stages.contains(stage) {
            violations.push(violation(
                "required_stage_missing",
                Some(stage),
                InvariantSeverity::Hard,
                format!("required VCF stage `{stage}` is missing"),
            ));
        }
        if !profile.defaults.params.contains_key(&stage_id) {
            violations.push(violation(
                "required_params_missing",
                Some(stage),
                InvariantSeverity::Hard,
                format!("missing typed params for VCF stage `{stage}`"),
            ));
        }
    }
    if profile.stability == StabilityTier::Stable && !stages.contains(id_catalog::VCF_FILTER) {
        violations.push(violation(
            "production_filter_required",
            Some(id_catalog::VCF_FILTER),
            InvariantSeverity::Hard,
            "production VCF profile must include vcf.filter stage",
        ));
    }

    if !profile
        .capabilities
        .required_metrics
        .contains(&"vcf.metrics")
    {
        violations.push(violation(
            "required_metrics_missing",
            None,
            InvariantSeverity::Hard,
            "VCF profile must emit `vcf.metrics`",
        ));
    }

    for artifact in [
        "report.json",
        "run_manifest.json",
        "tool_provenance.json",
        "invariants_report.json",
    ] {
        if !profile.capabilities.required_artifacts.contains(&artifact) {
            violations.push(violation(
                "required_artifact_missing",
                None,
                InvariantSeverity::Hard,
                format!("VCF profile must emit {artifact}"),
            ));
        }
    }

    for stage in [
        id_catalog::VCF_CALL,
        id_catalog::VCF_FILTER,
        id_catalog::VCF_STATS,
    ] {
        let stage_id = StageId::from_static(stage);
        let tool_id = profile
            .defaults
            .tools
            .get(&stage_id)
            .map(|t| t.as_str())
            .unwrap_or_default();
        if tool_id.is_empty() {
            violations.push(violation(
                "tool_pin_missing",
                Some(stage),
                InvariantSeverity::Hard,
                "VCF stage must have pinned tool selection",
            ));
        }
    }

    if let Some(DefaultParams::Vcf(VcfEffectiveParams::Call(call))) = profile
        .defaults
        .params
        .get(&StageId::from_static(id_catalog::VCF_CALL))
    {
        if call.sample_name.trim().is_empty() {
            violations.push(violation(
                "sample_name_missing",
                Some(id_catalog::VCF_CALL),
                InvariantSeverity::Hard,
                "vcf.call requires sample_name",
            ));
        }
        if profile.stability == StabilityTier::Stable && call.reference_fasta.is_none() {
            violations.push(violation(
                "reference_required",
                Some(id_catalog::VCF_CALL),
                InvariantSeverity::Hard,
                "production VCF profile requires reference_fasta",
            ));
        }
    }
    if let Some(DefaultParams::Vcf(VcfEffectiveParams::Filter(filter))) = profile
        .defaults
        .params
        .get(&StageId::from_static(id_catalog::VCF_FILTER))
    {
        if filter.sample_name.trim().is_empty() {
            violations.push(violation(
                "sample_name_missing",
                Some(id_catalog::VCF_FILTER),
                InvariantSeverity::Hard,
                "vcf.filter requires sample_name",
            ));
        }
        if profile.stability == StabilityTier::Stable && !filter.require_pass {
            violations.push(violation(
                "filter_stage_misconfigured",
                Some(id_catalog::VCF_FILTER),
                InvariantSeverity::Hard,
                "production VCF profile cannot disable pass-filtering",
            ));
        }
        if filter.require_bgzip_tabix
            && !profile.capabilities.required_artifacts.contains(&"vcf.tbi")
        {
            violations.push(violation(
                "artifact_correctness_missing",
                Some(id_catalog::VCF_FILTER),
                InvariantSeverity::Hard,
                "indexed VCF output requires vcf.tbi artifact declaration",
            ));
        }
    }
    if let Some(DefaultParams::Vcf(VcfEffectiveParams::Stats(stats))) = profile
        .defaults
        .params
        .get(&StageId::from_static(id_catalog::VCF_STATS))
    {
        if stats.sample_name.trim().is_empty() {
            violations.push(violation(
                "sample_name_missing",
                Some(id_catalog::VCF_STATS),
                InvariantSeverity::Hard,
                "vcf.stats requires sample_name",
            ));
        }
    }

    VcfProfileValidationReport {
        profile_id: profile.id.as_str().to_string(),
        invariants_version: VCF_INVARIANTS,
        valid: violations.is_empty(),
        violations,
    }
}
