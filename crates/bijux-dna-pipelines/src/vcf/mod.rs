use std::collections::BTreeSet;

use bijux_dna_core::ids::{
    AssayKind, LibraryLayout, LibraryModel, PlatformHint, StageId, ToolId, UdgTreatment,
};
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_vcf::params::{
    VcfCallParams, VcfEffectiveParams, VcfFilterParams, VcfStatsParams,
};
use serde::Serialize;

use crate::{
    ArtifactType, DefaultParams, Domain, EffectiveDefaults, InvariantSeverity,
    InvariantViolationV1, InvariantsPreset, InvariantsReportV1, MetricsBundle,
    PipelineCapabilities, PipelineId, PipelineProfile, ReportSection, StabilityTier,
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
        .copied()
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

    VcfProfileValidationReport {
        profile_id: profile.id.as_str().to_string(),
        invariants_version: VCF_INVARIANTS,
        valid: violations.is_empty(),
        violations,
    }
}

#[must_use]
pub fn vcf_minimal_profile() -> PipelineProfile {
    let mut defaults = EffectiveDefaults::default();

    defaults.tools.insert(
        StageId::from_static(id_catalog::VCF_CALL),
        ToolId::from_static(id_catalog::TOOL_BCFTOOLS),
    );
    defaults.tools.insert(
        StageId::from_static(id_catalog::VCF_FILTER),
        ToolId::from_static(id_catalog::TOOL_BCFTOOLS),
    );
    defaults.tools.insert(
        StageId::from_static(id_catalog::VCF_STATS),
        ToolId::from_static(id_catalog::TOOL_BCFTOOLS),
    );

    defaults.params.insert(
        StageId::from_static(id_catalog::VCF_CALL),
        DefaultParams::Vcf(VcfEffectiveParams::Call(VcfCallParams::default())),
    );
    defaults.params.insert(
        StageId::from_static(id_catalog::VCF_FILTER),
        DefaultParams::Vcf(VcfEffectiveParams::Filter(VcfFilterParams::default())),
    );
    defaults.params.insert(
        StageId::from_static(id_catalog::VCF_STATS),
        DefaultParams::Vcf(VcfEffectiveParams::Stats(VcfStatsParams::default())),
    );

    for stage in [
        id_catalog::VCF_CALL,
        id_catalog::VCF_FILTER,
        id_catalog::VCF_STATS,
    ] {
        defaults.rationales.insert(
            StageId::from_static(stage),
            "vcf minimal default".to_string(),
        );
    }

    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_VCF_MINIMAL),
        description: "Minimal VCF experimental profile",
        stability: StabilityTier::Experimental,
        input_domains: vec![Domain::Vcf],
        output_domains: vec![Domain::Vcf],
        defaults,
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: Some(InvariantsPreset::VcfMinimal),
        library_model: LibraryModel {
            layout: LibraryLayout::SingleEnd,
            udg_treatment: UdgTreatment::Unknown,
            platform_hint: PlatformHint::Unknown,
            assay_kind: AssayKind::Unknown,
        },
        capabilities: PipelineCapabilities {
            input_domains: vec![Domain::Vcf],
            output_domains: vec![Domain::Vcf],
            input_artifacts: vec![ArtifactType::ReportJson],
            output_artifacts: vec![ArtifactType::ReportJson, ArtifactType::MetricsBundle],
            required_inputs: vec!["vcf"],
            produces_outputs: vec!["vcf", "vcf.metrics"],
            report_sections: vec!["vcf"],
            required_report_sections: vec![ReportSection::Vcf, ReportSection::PipelineDefaults],
            required_metrics_bundles: vec![MetricsBundle::VcfCore],
            required_stages: vec![
                id_catalog::VCF_CALL,
                id_catalog::VCF_FILTER,
                id_catalog::VCF_STATS,
            ],
            required_metrics: vec!["vcf.metrics"],
            required_artifacts: vec![
                "report.json",
                "run_manifest.json",
                "tool_provenance.json",
                "invariants_report.json",
            ],
            supports_benchmarks: false,
        },
    }
}

#[must_use]
pub fn vcf_reference_basic_profile() -> PipelineProfile {
    let mut profile = vcf_minimal_profile();
    profile.id = PipelineId::from_static(id_catalog::PIPELINE_VCF_REFERENCE_BASIC);
    profile.description = "Reference-grade VCF baseline profile";
    profile.stability = StabilityTier::Beta;
    profile
}
