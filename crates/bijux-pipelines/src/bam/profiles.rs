//! BAM pipeline profiles and default params.

use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use bijux_domain_bam::defaults::{
    adna_capture_params_json, adna_shotgun_params_json, default_params_json,
};
use bijux_domain_bam::BamStage;

use crate::{
    ArtifactType, Domain, EffectiveDefaults, MetricsBundle, PipelineCapabilities, PipelineId,
    PipelineProfile, ReportSection, StabilityTier,
};

#[derive(Debug, Clone)]
struct BamStageDefault {
    stage: BamStage,
    params: serde_json::Value,
}

fn defaults_for(
    stages: &[BamStage],
    params_for_stage: fn(BamStage) -> serde_json::Value,
) -> Vec<BamStageDefault> {
    stages
        .iter()
        .map(|stage| BamStageDefault {
            stage: *stage,
            params: params_for_stage(*stage),
        })
        .collect()
}

fn to_effective_defaults(defaults: &[BamStageDefault]) -> EffectiveDefaults {
    let tools = BTreeMap::new();
    let mut params = BTreeMap::new();
    let mut rationales = BTreeMap::new();
    for entry in defaults {
        params.insert(entry.stage.as_str().to_string(), entry.params.clone());
        rationales.insert(
            entry.stage.as_str().to_string(),
            "pipeline default".to_string(),
        );
    }
    EffectiveDefaults {
        tools,
        params,
        rationales,
    }
}

fn filter_downstream(stages: &mut Vec<BamStage>) {
    if cfg!(feature = "bam_downstream") {
        return;
    }
    stages.retain(|stage| {
        !matches!(
            stage,
            BamStage::Haplogroups | BamStage::Genotyping | BamStage::Kinship
        )
    });
}

fn stable_bam_stages() -> Vec<BamStage> {
    vec![
        BamStage::Validate,
        BamStage::QcPre,
        BamStage::Filter,
        BamStage::Coverage,
        BamStage::Damage,
    ]
}

#[must_use]
pub fn bam_default_profile() -> PipelineProfile {
    let stages = stable_bam_stages();
    let defaults = defaults_for(&stages, default_params_json);
    let required_stages: Vec<&'static str> = stages.iter().map(|stage| stage.as_str()).collect();
    PipelineProfile {
        id: PipelineId::new("bam-to-bam__default__v1"),
        description: "Default BAM pipeline",
        stability: StabilityTier::Stable,
        input_domains: vec![Domain::Bam],
        output_domains: vec![Domain::Bam],
        defaults: to_effective_defaults(&defaults),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
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
            required_artifacts: vec!["report.json", "run_manifest.json", "stage_summaries.json"],
            supports_benchmarks: true,
        },
    }
}

#[must_use]
pub fn bam_adna_shotgun_profile() -> PipelineProfile {
    let mut stages = BamStage::all().to_vec();
    stages.retain(|stage| *stage != BamStage::Align);
    stages.retain(|stage| *stage != BamStage::Recalibration);
    filter_downstream(&mut stages);
    let defaults = defaults_for(&stages, adna_shotgun_params_json);
    let required_stages: Vec<&'static str> = stages.iter().map(|stage| stage.as_str()).collect();
    PipelineProfile {
        id: PipelineId::new("bam-to-bam__adna_shotgun__v1"),
        description: "Ancient DNA shotgun defaults",
        stability: StabilityTier::Beta,
        input_domains: vec![Domain::Bam],
        output_domains: vec![Domain::Bam],
        defaults: to_effective_defaults(&defaults),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: Some("adna"),
        capabilities: PipelineCapabilities {
            input_domains: vec![Domain::Bam],
            output_domains: vec![Domain::Bam],
            input_artifacts: vec![ArtifactType::Bam],
            output_artifacts: vec![ArtifactType::Bam, ArtifactType::MetricsBundle],
            required_inputs: vec!["bam"],
            produces_outputs: vec!["bam", "bam.metrics"],
            report_sections: vec!["bam"],
            required_report_sections: vec![ReportSection::Bam, ReportSection::PipelineDefaults],
            required_metrics_bundles: vec![MetricsBundle::BamAdna],
            required_stages,
            required_metrics: vec!["bam.metrics"],
            required_artifacts: vec!["report.json", "run_manifest.json", "stage_summaries.json"],
            supports_benchmarks: true,
        },
    }
}

#[must_use]
pub fn bam_adna_capture_profile() -> PipelineProfile {
    let mut stages = BamStage::all().to_vec();
    stages.retain(|stage| *stage != BamStage::Align);
    stages.retain(|stage| *stage != BamStage::Recalibration);
    filter_downstream(&mut stages);
    let defaults = defaults_for(&stages, adna_capture_params_json);
    let required_stages: Vec<&'static str> = stages.iter().map(|stage| stage.as_str()).collect();
    let profile = PipelineProfile {
        id: PipelineId::new("bam-to-bam__adna_capture__v1"),
        description: "Ancient DNA capture defaults",
        stability: StabilityTier::Beta,
        input_domains: vec![Domain::Bam],
        output_domains: vec![Domain::Bam],
        defaults: to_effective_defaults(&defaults),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: Some("adna"),
        capabilities: PipelineCapabilities {
            input_domains: vec![Domain::Bam],
            output_domains: vec![Domain::Bam],
            input_artifacts: vec![ArtifactType::Bam],
            output_artifacts: vec![ArtifactType::Bam, ArtifactType::MetricsBundle],
            required_inputs: vec!["bam"],
            produces_outputs: vec!["bam", "bam.metrics"],
            report_sections: vec!["bam"],
            required_report_sections: vec![ReportSection::Bam, ReportSection::PipelineDefaults],
            required_metrics_bundles: vec![MetricsBundle::BamAdna],
            required_stages,
            required_metrics: vec!["bam.metrics"],
            required_artifacts: vec!["report.json", "run_manifest.json", "stage_summaries.json"],
            supports_benchmarks: true,
        },
    };
    profile
}

/// # Errors
/// Returns an error if the requested profile id is unknown.
pub fn bam_profiles_by_id(id: &str) -> Result<PipelineProfile> {
    match id {
        "bam-to-bam__default__v1" => Ok(bam_default_profile()),
        "bam-to-bam__adna_shotgun__v1" => Ok(bam_adna_shotgun_profile()),
        "bam-to-bam__adna_capture__v1" => Ok(bam_adna_capture_profile()),
        _ => Err(anyhow!("unknown BAM profile: {id}")),
    }
}
