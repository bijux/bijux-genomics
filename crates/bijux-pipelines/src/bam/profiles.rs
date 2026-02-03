//! BAM pipeline profiles and default params.

use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use bijux_domain_bam::params::{
    BamEffectiveParams, ContaminationScope, DamageEffectiveParams, UdgModel,
};
use bijux_domain_bam::stage_spec;
use bijux_domain_bam::BamStage;

use crate::{
    Domain, EffectiveDefaults, PipelineCapabilities, PipelineId, PipelineProfile, StageNode,
    StabilityTier,
};

#[derive(Debug, Clone)]
struct BamStageDefault {
    stage: BamStage,
    tool: &'static str,
    params: BamEffectiveParams,
}

fn base_defaults() -> Vec<BamStageDefault> {
    BamStage::all()
        .iter()
        .map(|stage| {
            let spec = stage_spec(*stage);
            BamStageDefault {
                stage: *stage,
                tool: spec.default_tool,
                params: spec.default_params,
            }
        })
        .collect()
}

fn to_effective_defaults(defaults: &[BamStageDefault]) -> EffectiveDefaults {
    let mut tools = BTreeMap::new();
    let mut params = BTreeMap::new();
    for entry in defaults {
        tools.insert(entry.stage.as_str().to_string(), entry.tool.to_string());
        params.insert(
            entry.stage.as_str().to_string(),
            bam_params_value(&entry.params),
        );
    }
    EffectiveDefaults { tools, params }
}

fn bam_params_value(params: &BamEffectiveParams) -> serde_json::Value {
    match params {
        BamEffectiveParams::Align(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::Validate(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::QcPre(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::Filter(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::Markdup(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::Complexity(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::Coverage(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::Damage(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::Authenticity(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::Contamination(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::Sex(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::BiasMitigation(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::Recalibration(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::Haplogroups(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::Genotyping(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::Kinship(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
    }
}

fn to_graph(stages: &[BamStage]) -> Vec<StageNode> {
    stages
        .iter()
        .map(|stage| StageNode {
            stage_id: stage.as_str().to_string(),
        })
        .collect()
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

fn filter_defaults(defaults: &mut Vec<BamStageDefault>, stages: &[BamStage]) {
    let allowed: std::collections::HashSet<_> = stages.iter().copied().collect();
    defaults.retain(|entry| allowed.contains(&entry.stage));
}

#[must_use]
pub fn bam_default_profile() -> PipelineProfile {
    let mut defaults = base_defaults();
    let mut stages = BamStage::all().to_vec();
    stages.retain(|stage| *stage != BamStage::Align);
    filter_downstream(&mut stages);
    filter_defaults(&mut defaults, &stages);
    PipelineProfile {
        id: PipelineId::new("bam__default__v1"),
        description: "Default BAM pipeline",
        stability: StabilityTier::Stable,
        domains: vec![Domain::Bam],
        graph: to_graph(&stages),
        defaults: to_effective_defaults(&defaults),
        invariants_preset: None,
        capabilities: PipelineCapabilities {
            required_inputs: vec!["bam"],
            produces_outputs: vec!["bam", "bam.metrics"],
            report_sections: vec!["bam"],
            supports_benchmarking: true,
        },
    }
}

#[must_use]
pub fn bam_adna_shotgun_profile() -> PipelineProfile {
    let mut defaults = base_defaults();
    let mut stages = BamStage::all().to_vec();
    stages.retain(|stage| *stage != BamStage::Align);
    stages.retain(|stage| *stage != BamStage::Recalibration);
    filter_downstream(&mut stages);
    filter_defaults(&mut defaults, &stages);
    for entry in &mut defaults {
        match entry.stage {
            BamStage::Filter => {
                if let BamEffectiveParams::Filter(params) = &mut entry.params {
                    params.min_length = 30;
                    params.mapq_threshold = 30;
                }
            }
            BamStage::Damage => {
                if let BamEffectiveParams::Damage(params) = &mut entry.params {
                    *params = DamageEffectiveParams {
                        udg_model: UdgModel::NonUdg,
                        pmd_threshold_5p: 0.3,
                        pmd_threshold_3p: 0.3,
                        trim_5p: 2,
                        trim_3p: 2,
                    };
                }
            }
            BamStage::Contamination => {
                if let BamEffectiveParams::Contamination(params) = &mut entry.params {
                    params.scope = ContaminationScope::Both;
                }
            }
            _ => {}
        }
    }
    PipelineProfile {
        id: PipelineId::new("bam__adna_shotgun__v1"),
        description: "Ancient DNA shotgun defaults",
        stability: StabilityTier::Beta,
        domains: vec![Domain::Bam],
        graph: to_graph(&stages),
        defaults: to_effective_defaults(&defaults),
        invariants_preset: Some("adna"),
        capabilities: PipelineCapabilities {
            required_inputs: vec!["bam"],
            produces_outputs: vec!["bam", "bam.metrics"],
            report_sections: vec!["bam"],
            supports_benchmarking: true,
        },
    }
}

#[must_use]
pub fn bam_adna_capture_profile() -> PipelineProfile {
    let mut profile = bam_adna_shotgun_profile();
    profile.id = PipelineId::new("bam__adna_capture__v1");
    profile.description = "Ancient DNA capture defaults";
    for (stage_id, params) in profile.defaults.params.iter_mut() {
        if stage_id == "bam.filter" {
            if let Ok(mut filter) = serde_json::from_value::<
                bijux_domain_bam::params::FilterEffectiveParams,
            >(params.clone())
            {
                filter.min_length = 25;
                filter.mapq_threshold = 30;
                if let Ok(value) = serde_json::to_value(&filter) {
                    *params = value;
                }
            }
        }
    }
    profile
}

/// # Errors
/// Returns an error if the requested profile id is unknown.
pub fn bam_profiles_by_id(id: &str) -> Result<PipelineProfile> {
    match id {
        "bam__default__v1" => Ok(bam_default_profile()),
        "bam__adna_shotgun__v1" => Ok(bam_adna_shotgun_profile()),
        "bam__adna_capture__v1" => Ok(bam_adna_capture_profile()),
        _ => Err(anyhow!("unknown BAM profile: {id}")),
    }
}
