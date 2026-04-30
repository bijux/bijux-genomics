//! VCF profile definitions.

use bijux_dna_core::ids::{
    AssayKind, LibraryLayout, LibraryModel, PlatformHint, StageId, ToolId, UdgTreatment,
};
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_vcf::params::{
    VcfCallParams, VcfEffectiveParams, VcfFilterParams, VcfStatsParams,
};

use crate::{
    ArtifactType, DefaultParams, Domain, EffectiveDefaults, InvariantsPreset, MetricsBundle,
    PipelineCapabilities, PipelineId, PipelineProfile, ReportSection, StabilityTier,
};

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

    for stage in [id_catalog::VCF_CALL, id_catalog::VCF_FILTER, id_catalog::VCF_STATS] {
        defaults.rationales.insert(StageId::from_static(stage), "vcf minimal default".to_string());
    }

    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_VCF_MINIMAL),
        description: "Minimal VCF experimental profile",
        stability: StabilityTier::Beta,
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
            required_inputs: vec!["vcf", "sample_name"],
            produces_outputs: vec!["vcf", "vcf.metrics"],
            report_sections: vec!["vcf"],
            required_report_sections: vec![ReportSection::Vcf, ReportSection::PipelineDefaults],
            required_metrics_bundles: vec![MetricsBundle::VcfCore],
            required_stages: vec![
                id_catalog::VCF_CALL.to_string(),
                id_catalog::VCF_FILTER.to_string(),
                id_catalog::VCF_STATS.to_string(),
            ],
            required_metrics: vec!["vcf.metrics"],
            required_artifacts: vec![
                "report.json",
                "run_manifest.json",
                "tool_provenance.json",
                "invariants_report.json",
                "vcf.tbi",
            ],
            supports_benchmarks: false,
            supports_sample_sheet: false,
            workflow_template_ids: Vec::new(),
            batch_semantics: None,
            fan_artifact_rules: Vec::new(),
            failure_policy: Vec::new(),
            evidence_summary: None,
            parameter_policy: None,
        },
    }
}

#[must_use]
pub fn vcf_reference_basic_profile() -> PipelineProfile {
    let mut profile = vcf_minimal_profile();
    profile.id = PipelineId::from_static(id_catalog::PIPELINE_VCF_REFERENCE_BASIC);
    profile.description = "Reference-grade VCF baseline profile";
    profile.stability = StabilityTier::Stable;
    if let Some(DefaultParams::Vcf(VcfEffectiveParams::Call(call))) =
        profile.defaults.params.get_mut(&StageId::from_static(id_catalog::VCF_CALL))
    {
        call.reference_fasta = Some("reference.fa".to_string());
        call.sample_name = "reference_sample".to_string();
    }
    if let Some(DefaultParams::Vcf(VcfEffectiveParams::Filter(filter))) =
        profile.defaults.params.get_mut(&StageId::from_static(id_catalog::VCF_FILTER))
    {
        filter.production_profile = true;
        filter.require_pass = true;
    }
    profile
}
