//! BAM pipeline profiles and default params.

use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_bam::defaults::{
    adna_capture_params_json, adna_shotgun_params_json, default_params_json,
};
use bijux_dna_domain_bam::params::BamEffectiveParams;
use bijux_dna_domain_bam::BamStage;

use crate::{
    ArtifactType, DefaultParams, Domain, EffectiveDefaults, MetricsBundle, PipelineCapabilities,
    PipelineId, PipelineProfile, ReportSection, StabilityTier,
};

#[derive(Debug, Clone)]
struct BamStageDefault {
    stage: BamStage,
    params: BamEffectiveParams,
}

fn defaults_for(
    stages: &[BamStage],
    params_for_stage: fn(BamStage) -> serde_json::Value,
) -> Vec<BamStageDefault> {
    stages
        .iter()
        .map(|stage| BamStageDefault {
            stage: *stage,
            params: stage
                .parse_effective_params(&params_for_stage(*stage))
                .unwrap_or_else(|err| {
                    panic!(
                        "failed to parse typed BAM defaults for stage {}: {err}",
                        stage.as_str()
                    )
                }),
        })
        .collect()
}

fn to_effective_defaults(defaults: &[BamStageDefault]) -> EffectiveDefaults {
    let mut tools = BTreeMap::new();
    let mut params = BTreeMap::new();
    let mut rationales = BTreeMap::new();
    for entry in defaults {
        let default_tool = match entry.stage {
            BamStage::Align => id_catalog::TOOL_BWA,
            BamStage::Validate => id_catalog::TOOL_SAMTOOLS,
            BamStage::QcPre => id_catalog::TOOL_SAMTOOLS,
            BamStage::Filter => id_catalog::TOOL_SAMTOOLS,
            BamStage::Markdup => id_catalog::TOOL_GATK,
            BamStage::Complexity => id_catalog::TOOL_PRESEQ,
            BamStage::Coverage => id_catalog::TOOL_MOSDEPTH,
            BamStage::Damage => id_catalog::TOOL_PYDAMAGE,
            BamStage::Authenticity => id_catalog::TOOL_AUTHENTICCT,
            BamStage::Contamination => id_catalog::TOOL_AUTHENTICCT,
            BamStage::Sex => id_catalog::TOOL_RXY,
            BamStage::BiasMitigation => id_catalog::TOOL_ANGSD,
            BamStage::Recalibration => id_catalog::TOOL_GATK,
            BamStage::Haplogroups => id_catalog::TOOL_YLEAF,
            BamStage::Genotyping => id_catalog::TOOL_ANGSD,
            BamStage::Kinship => id_catalog::TOOL_KING,
        };
        tools.insert(
            StageId::from_static(entry.stage.as_str()),
            ToolId::from_static(default_tool),
        );
        params.insert(
            StageId::from_static(entry.stage.as_str()),
            DefaultParams::Bam(entry.params.clone()),
        );
        rationales.insert(
            StageId::from_static(entry.stage.as_str()),
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

fn bam_stage_order(stage: &BamStage) -> usize {
    match stage {
        BamStage::Align => 0,
        BamStage::Validate => 1,
        BamStage::QcPre => 2,
        BamStage::Filter => 3,
        BamStage::Markdup => 4,
        BamStage::Complexity => 5,
        BamStage::Coverage => 6,
        BamStage::Damage => 7,
        BamStage::Authenticity => 8,
        BamStage::Contamination => 9,
        BamStage::Sex => 10,
        BamStage::BiasMitigation => 11,
        BamStage::Recalibration => 12,
        BamStage::Haplogroups => 13,
        BamStage::Genotyping => 14,
        BamStage::Kinship => 15,
    }
}

fn catalog_bam_stages() -> Vec<BamStage> {
    let parsed: toml::Value = include_str!("../../../../configs/stages.toml")
        .parse()
        .expect("generated configs/stages.toml must parse");
    let mut stages = parsed
        .get("stages")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| {
            entry
                .get("id")
                .and_then(toml::Value::as_str)
                .and_then(|id| {
                    if id.starts_with("bam.") {
                        BamStage::try_from(id).ok()
                    } else {
                        None
                    }
                })
        })
        .collect::<Vec<_>>();
    stages.sort_by_key(bam_stage_order);
    stages.dedup();
    stages
}

#[must_use]
pub fn bam_default_profile() -> PipelineProfile {
    let stages = stable_bam_stages();
    let defaults = defaults_for(&stages, default_params_json);
    let required_stages: Vec<&'static str> = stages.iter().map(|stage| stage.as_str()).collect();
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_BAM_DEFAULT),
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
    let mut stages = catalog_bam_stages();
    stages.retain(|stage| *stage != BamStage::Align);
    stages.retain(|stage| *stage != BamStage::Recalibration);
    filter_downstream(&mut stages);
    let defaults = defaults_for(&stages, adna_shotgun_params_json);
    let required_stages: Vec<&'static str> = stages.iter().map(|stage| stage.as_str()).collect();
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_BAM_ADNA_SHOTGUN),
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
    let mut stages = catalog_bam_stages();
    stages.retain(|stage| *stage != BamStage::Align);
    stages.retain(|stage| *stage != BamStage::Recalibration);
    filter_downstream(&mut stages);
    let defaults = defaults_for(&stages, adna_capture_params_json);
    let required_stages: Vec<&'static str> = stages.iter().map(|stage| stage.as_str()).collect();
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_BAM_ADNA_CAPTURE),
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
    }
}

/// # Errors
/// Returns an error if the requested profile id is unknown.
pub fn bam_profiles_by_id(id: &str) -> Result<PipelineProfile> {
    match id {
        id_catalog::PIPELINE_BAM_DEFAULT => Ok(bam_default_profile()),
        id_catalog::PIPELINE_BAM_ADNA_SHOTGUN => Ok(bam_adna_shotgun_profile()),
        id_catalog::PIPELINE_BAM_ADNA_CAPTURE => Ok(bam_adna_capture_profile()),
        _ => Err(anyhow!("unknown BAM profile: {id}")),
    }
}
