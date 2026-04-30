use crate::params::{
    BamEffectiveParams, ContaminationEffectiveParams, ContaminationScope, DamageEffectiveParams,
    FilterEffectiveParams, UdgModel,
};
use crate::stage_specs::stage_spec;
use crate::BamStage;
use serde::Serialize;

#[must_use]
pub fn default_params_json(stage: BamStage) -> serde_json::Value {
    let spec = stage_spec(stage);
    bam_params_value(&spec.default_params)
}

#[must_use]
pub fn adna_shotgun_params_json(stage: BamStage) -> serde_json::Value {
    let mut value = default_params_json(stage);
    match stage {
        BamStage::Align => {
            if let Ok(mut params) = serde_json::from_value::<BamEffectiveParams>(value.clone()) {
                if let BamEffectiveParams::Align(ref mut align) = params {
                    align.preset = "adna_short".to_string();
                    align.strategy_id = "bwa_aln_adna_short".to_string();
                    align.mode = "seeded_short_read".to_string();
                }
                if let Ok(updated) = serde_json::to_value(params) {
                    value = updated;
                }
            }
        }
        BamStage::Filter => {
            if let Ok(mut params) = serde_json::from_value::<FilterEffectiveParams>(value.clone()) {
                params.min_length = 30;
                params.mapq_threshold = 30;
                if let Ok(updated) = serde_json::to_value(params) {
                    value = updated;
                }
            }
        }
        BamStage::Damage => {
            let params = DamageEffectiveParams {
                udg_model: UdgModel::NonUdg,
                pmd_threshold_5p: 0.3,
                pmd_threshold_3p: 0.3,
                trim_5p: 2,
                trim_3p: 2,
                damage_tool_profile: Some("ancient_dna_evidence".to_string()),
                evidence_only: true,
            };
            if let Ok(updated) = serde_json::to_value(params) {
                value = updated;
            }
        }
        BamStage::Contamination => {
            if let Ok(mut params) =
                serde_json::from_value::<ContaminationEffectiveParams>(value.clone())
            {
                params.scope = ContaminationScope::Both;
                if let Ok(updated) = serde_json::to_value(params) {
                    value = updated;
                }
            }
        }
        _ => {}
    }
    value
}

#[must_use]
pub fn adna_capture_params_json(stage: BamStage) -> serde_json::Value {
    let mut value = adna_shotgun_params_json(stage);
    if stage == BamStage::Filter {
        if let Ok(mut params) = serde_json::from_value::<FilterEffectiveParams>(value.clone()) {
            params.min_length = 25;
            params.mapq_threshold = 30;
            if let Ok(updated) = serde_json::to_value(params) {
                value = updated;
            }
        }
    }
    value
}

#[allow(clippy::match_same_arms)]
fn bam_params_value(params: &BamEffectiveParams) -> serde_json::Value {
    match params {
        BamEffectiveParams::Align(inner) => params_value(inner),
        BamEffectiveParams::Validate(inner) => params_value(inner),
        BamEffectiveParams::QcPre(inner) => params_value(inner),
        BamEffectiveParams::MappingSummary(inner) => params_value(inner),
        BamEffectiveParams::Filter(inner) => params_value(inner),
        BamEffectiveParams::MapqFilter(inner) => params_value(inner),
        BamEffectiveParams::LengthFilter(inner) => params_value(inner),
        BamEffectiveParams::Markdup(inner) => params_value(inner),
        BamEffectiveParams::DuplicationMetrics(inner) => params_value(inner),
        BamEffectiveParams::Complexity(inner) => params_value(inner),
        BamEffectiveParams::Coverage(inner) => params_value(inner),
        BamEffectiveParams::InsertSize(inner) => params_value(inner),
        BamEffectiveParams::GcBias(inner) => params_value(inner),
        BamEffectiveParams::EndogenousContent(inner) => params_value(inner),
        BamEffectiveParams::OverlapCorrection(inner) => params_value(inner),
        BamEffectiveParams::Damage(inner) => params_value(inner),
        BamEffectiveParams::Authenticity(inner) => params_value(inner),
        BamEffectiveParams::Contamination(inner) => params_value(inner),
        BamEffectiveParams::Sex(inner) => params_value(inner),
        BamEffectiveParams::BiasMitigation(inner) => params_value(inner),
        BamEffectiveParams::Recalibration(inner) => params_value(inner),
        BamEffectiveParams::Haplogroups(inner) => params_value(inner),
        BamEffectiveParams::Genotyping(inner) => params_value(inner),
        BamEffectiveParams::Kinship(inner) => params_value(inner),
    }
}

fn params_value<T: Serialize>(params: &T) -> serde_json::Value {
    match serde_json::to_value(params) {
        Ok(value) => value,
        Err(err) => unreachable!("BAM default params must serialize: {err}"),
    }
}
