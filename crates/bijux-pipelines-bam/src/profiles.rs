//! BAM pipeline profiles and default params.

use anyhow::{anyhow, Result};
use bijux_domain_bam::params::{
    BamEffectiveParams, ContaminationScope, DamageEffectiveParams, UdgModel,
};
use bijux_domain_bam::stage_spec;
use bijux_domain_bam::BamStage;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BamStageDefaults {
    pub stage: BamStage,
    pub tool: &'static str,
    pub params: BamEffectiveParams,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamPipelineProfile {
    pub id: &'static str,
    pub description: &'static str,
    pub stages: Vec<BamStage>,
    pub defaults: Vec<BamStageDefaults>,
}

impl BamPipelineProfile {
    #[must_use]
    pub fn default_params(&self, stage: BamStage) -> Option<&BamEffectiveParams> {
        self.defaults
            .iter()
            .find(|entry| entry.stage == stage)
            .map(|entry| &entry.params)
    }

    #[must_use]
    pub fn default_tool(&self, stage: BamStage) -> Option<&'static str> {
        self.defaults
            .iter()
            .find(|entry| entry.stage == stage)
            .map(|entry| entry.tool)
    }
}

#[must_use]
pub fn bam_default_profile() -> BamPipelineProfile {
    let defaults = BamStage::all()
        .iter()
        .map(|stage| {
            let spec = stage_spec(*stage);
            BamStageDefaults {
                stage: *stage,
                tool: spec.default_tool,
                params: spec.default_params,
            }
        })
        .collect();
    BamPipelineProfile {
        id: "default",
        description: "Default BAM pipeline",
        stages: BamStage::all().to_vec(),
        defaults,
    }
}

#[must_use]
pub fn bam_adna_shotgun_profile() -> BamPipelineProfile {
    let mut profile = bam_default_profile();
    profile.id = "adna-shotgun";
    profile.description = "Ancient DNA shotgun defaults";
    profile
        .stages
        .retain(|stage| *stage != BamStage::Recalibration);
    for entry in &mut profile.defaults {
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
    profile
}

#[must_use]
pub fn bam_adna_capture_profile() -> BamPipelineProfile {
    let mut profile = bam_adna_shotgun_profile();
    profile.id = "adna-capture";
    profile.description = "Ancient DNA capture defaults";
    for entry in &mut profile.defaults {
        if entry.stage == BamStage::Filter {
            if let BamEffectiveParams::Filter(params) = &mut entry.params {
                params.min_length = 25;
                params.mapq_threshold = 30;
            }
        }
    }
    profile
}

/// # Errors
/// Returns an error if the requested profile id is unknown.
pub fn profile_by_id(id: &str) -> Result<BamPipelineProfile> {
    match id {
        "default" => Ok(bam_default_profile()),
        "adna-shotgun" => Ok(bam_adna_shotgun_profile()),
        "adna-capture" => Ok(bam_adna_capture_profile()),
        _ => Err(anyhow!("unknown BAM profile: {id}")),
    }
}
