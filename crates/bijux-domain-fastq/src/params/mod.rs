//! Owner: bijux-domain-fastq
//! Canonical effective parameters for FASTQ stages.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    stage_id: &str,
    value: &serde_json::Value,
) -> Option<EffectiveParams> {
    match stage_id {
        "fastq.validate_pre" | "fastq.stats_neutral" | "fastq.correct" | "fastq.umi" => {
            serde_json::from_value::<validate::ValidateEffectiveParams>(value.clone())
                .ok()
                .map(EffectiveParams::Validate)
        }
        "fastq.trim" => serde_json::from_value::<trim::TrimEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Trim),
        "fastq.filter" => serde_json::from_value::<filter::FilterEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Filter),
        "fastq.merge" => serde_json::from_value::<merge::MergeEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Merge),
        "fastq.rrna" => serde_json::from_value::<rrna::RrnaEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Rrna),
        "fastq.screen" => serde_json::from_value::<screen::ScreenEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Screen),
        "fastq.qc_post" => serde_json::from_value::<qc_post::QcPostEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::QcPost),
        "fastq.preprocess" => {
            serde_json::from_value::<preprocess::PreprocessEffectiveParams>(value.clone())
                .ok()
                .map(EffectiveParams::Preprocess)
        }
        _ => None,
    }
}
