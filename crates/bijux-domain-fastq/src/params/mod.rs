//! Owner: bijux-domain-fastq
//! Canonical effective parameters for FASTQ stages.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::stages::ids::{
    STAGE_CORRECT, STAGE_DETECT_ADAPTERS, STAGE_FILTER, STAGE_MERGE, STAGE_PREPROCESS,
    STAGE_QC_POST, STAGE_RRNA, STAGE_SCREEN, STAGE_STATS_NEUTRAL, STAGE_TRIM, STAGE_UMI,
    STAGE_VALIDATE_PRE,
};
use bijux_core::ids::StageId;

pub mod defaults;
pub mod detect_adapters;
pub mod filter;
pub mod merge;
pub mod preprocess;
pub mod qc_post;
pub mod rrna;
pub mod screen;
pub mod trim;
pub mod validate;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PairedMode {
    SingleEnd,
    PairedEnd,
    Unknown,
}

impl PairedMode {
    #[must_use]
    pub fn from_has_r2(has_r2: bool) -> Self {
        if has_r2 {
            Self::PairedEnd
        } else {
            Self::SingleEnd
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "stage")]
pub enum EffectiveParams {
    Validate(validate::ValidateEffectiveParams),
    DetectAdapters(detect_adapters::DetectAdaptersEffectiveParams),
    Trim(trim::TrimEffectiveParams),
    Filter(filter::FilterEffectiveParams),
    Merge(merge::MergeEffectiveParams),
    Rrna(rrna::RrnaEffectiveParams),
    Screen(screen::ScreenEffectiveParams),
    QcPost(qc_post::QcPostEffectiveParams),
    Preprocess(preprocess::PreprocessEffectiveParams),
}

impl EffectiveParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        match self {
            Self::Validate(params) => params.missing_required_fields(),
            Self::DetectAdapters(params) => params.missing_required_fields(),
            Self::Trim(params) => params.missing_required_fields(),
            Self::Filter(params) => params.missing_required_fields(),
            Self::Merge(params) => params.missing_required_fields(),
            Self::Rrna(params) => params.missing_required_fields(),
            Self::Screen(params) => params.missing_required_fields(),
            Self::QcPost(params) => params.missing_required_fields(),
            Self::Preprocess(params) => params.missing_required_fields(),
        }
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        match self {
            Self::Validate(params) => params.retention_conditions(),
            Self::DetectAdapters(params) => params.retention_conditions(),
            Self::Trim(params) => params.retention_conditions(),
            Self::Filter(params) => params.retention_conditions(),
            Self::Merge(params) => params.retention_conditions(),
            Self::Rrna(params) => params.retention_conditions(),
            Self::Screen(params) => params.retention_conditions(),
            Self::QcPost(params) => params.retention_conditions(),
            Self::Preprocess(params) => params.retention_conditions(),
        }
    }
}

#[must_use]
pub fn parse_effective_params(
    stage_id: &StageId,
    value: &serde_json::Value,
) -> Option<EffectiveParams> {
    if stage_id == &STAGE_VALIDATE_PRE
        || stage_id == &STAGE_STATS_NEUTRAL
        || stage_id == &STAGE_CORRECT
        || stage_id == &STAGE_UMI
    {
        return serde_json::from_value::<validate::ValidateEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Validate);
    }
    if stage_id == &STAGE_DETECT_ADAPTERS {
        return serde_json::from_value::<detect_adapters::DetectAdaptersEffectiveParams>(
            value.clone(),
        )
        .ok()
        .map(EffectiveParams::DetectAdapters);
    }
    if stage_id == &STAGE_TRIM {
        return serde_json::from_value::<trim::TrimEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Trim);
    }
    if stage_id == &STAGE_FILTER {
        return serde_json::from_value::<filter::FilterEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Filter);
    }
    if stage_id == &STAGE_MERGE {
        return serde_json::from_value::<merge::MergeEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Merge);
    }
    if stage_id == &STAGE_RRNA {
        return serde_json::from_value::<rrna::RrnaEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Rrna);
    }
    if stage_id == &STAGE_SCREEN {
        return serde_json::from_value::<screen::ScreenEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Screen);
    }
    if stage_id == &STAGE_QC_POST {
        return serde_json::from_value::<qc_post::QcPostEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::QcPost);
    }
    if stage_id == &STAGE_PREPROCESS {
        return serde_json::from_value::<preprocess::PreprocessEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Preprocess);
    }
    None
}
