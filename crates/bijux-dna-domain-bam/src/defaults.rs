use crate::params::{
    BamEffectiveParams, ContaminationEffectiveParams, ContaminationScope, DamageEffectiveParams,
    FilterEffectiveParams, UdgModel,
};
use crate::stage_specs::stage_spec;
use crate::BamStage;

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
        BamEffectiveParams::MappingSummary(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::Filter(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::MapqFilter(inner) => {
            serde_json::to_value(inner).unwrap_or(serde_json::Value::Null)
        }
        BamEffectiveParams::LengthFilter(inner) => {
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
        BamEffectiveParams::EndogenousContent(inner) => {
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
