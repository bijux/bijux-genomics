//! Baseline BAM profile definitions.

use bijux_dna_core::ids::{AssayKind, LibraryLayout, LibraryModel, PlatformHint, UdgTreatment};
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_bam::defaults::default_params_json;

use super::profile_defaults::{defaults_for, stable_bam_stages, to_effective_defaults};
use crate::{
    ArtifactType, Domain, MetricsBundle, PipelineCapabilities, PipelineId, PipelineProfile,
    ReportSection, StabilityTier,
};

#[must_use]
pub fn bam_default_profile() -> PipelineProfile {
    let stages = stable_bam_stages();
    let defaults = defaults_for(&stages, default_params_json);
    let required_stages: Vec<String> =
        stages.iter().map(|stage| stage.as_str().to_string()).collect();
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_BAM_DEFAULT),
        description: "Default BAM pipeline",
        stability: StabilityTier::Stable,
        input_domains: vec![Domain::Bam],
        output_domains: vec![Domain::Bam],
        defaults: to_effective_defaults(&defaults),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        library_model: LibraryModel {
            layout: LibraryLayout::SingleEnd,
            udg_treatment: UdgTreatment::Unknown,
            platform_hint: PlatformHint::Illumina,
            assay_kind: AssayKind::Unknown,
        },
        capabilities: PipelineCapabilities {
            input_domains: vec![Domain::Bam],
            output_domains: vec![Domain::Bam],
            input_artifacts: vec![ArtifactType::Bam],
            output_artifacts: vec![ArtifactType::Bam, ArtifactType::MetricsBundle],
            required_inputs: vec!["bam"],
            produces_outputs: vec!["bam", "bam.metrics"],
            report_sections: vec!["bam"],
            required_report_sections: vec![ReportSection::Bam, ReportSection::PipelineDefaults],
            required_metrics_bundles: vec![MetricsBundle::BamCore],
            required_stages,
            required_metrics: vec!["bam.metrics"],
            required_artifacts: vec![
                "report.json",
                "run_manifest.json",
                "stage_summaries.json",
                "invariants_report.json",
            ],
            supports_benchmarks: true,
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
