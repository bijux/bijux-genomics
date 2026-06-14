use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::{edna, processing, quality};

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DamageMode {
    Ancient,
    UdgTrimmed,
}

impl DamageMode {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ancient => "ancient",
            Self::UdgTrimmed => "udg_trimmed",
        }
    }
}

impl FromStr for DamageMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ancient" => Ok(Self::Ancient),
            "udg_trimmed" => Ok(Self::UdgTrimmed),
            _ => Err(format!(
                "unsupported damage_mode `{value}`; expected one of: ancient, udg_trimmed"
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "stage")]
pub enum EffectiveParams {
    Validate(quality::validate::ValidateEffectiveParams),
    Stats(quality::stats::FastqStatsParams),
    ReadLengthProfile(quality::stats::FastqReadLengthProfileParams),
    EstimateLibraryComplexityPrealign(quality::stats::FastqEstimateLibraryComplexityPrealignParams),
    Correct(processing::correct::FastqCorrectParams),
    Umi(processing::umi::FastqUmiParams),
    DetectAdapters(quality::detect_adapters::DetectAdaptersEffectiveParams),
    Trim(quality::trim::TrimEffectiveParams),
    TrimTerminalDamage(quality::trim::TrimTerminalDamageParams),
    TrimPolygTails(quality::trim::TrimPolygTailsParams),
    RemoveDuplicates(processing::remove_duplicates::RemoveDuplicatesEffectiveParams),
    Filter(quality::filter::FilterEffectiveParams),
    Merge(processing::merge::MergeEffectiveParams),
    ReferenceIndex(processing::reference_index::ReferenceIndexEffectiveParams),
    HostDepletion(quality::screen::HostDepletionEffectiveParams),
    ReferenceContaminantDepletion(quality::screen::ReferenceContaminantEffectiveParams),
    Rrna(quality::screen::RrnaEffectiveParams),
    Screen(quality::screen::ScreenEffectiveParams),
    ReportQc(quality::qc_post::QcPostEffectiveParams),
    OverrepresentedProfile(quality::stats::FastqOverrepresentedProfileParams),
    PrimerNormalization(edna::PrimerNormalizationEffectiveParams),
    ChimeraDetection(edna::ChimeraDetectionEffectiveParams),
    AsvInference(edna::AsvInferenceEffectiveParams),
    OtuClustering(edna::OtuClusteringEffectiveParams),
    AbundanceNormalization(edna::AbundanceNormalizationEffectiveParams),
}

impl EffectiveParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        match self {
            Self::Validate(params) => params.missing_required_fields(),
            Self::Stats(params) => params.missing_required_fields(),
            Self::ReadLengthProfile(params) => params.missing_required_fields(),
            Self::EstimateLibraryComplexityPrealign(params) => params.missing_required_fields(),
            Self::Correct(params) => params.missing_required_fields(),
            Self::Umi(params) => params.missing_required_fields(),
            Self::DetectAdapters(params) => params.missing_required_fields(),
            Self::Trim(params) => params.missing_required_fields(),
            Self::TrimTerminalDamage(params) => params.missing_required_fields(),
            Self::TrimPolygTails(params) => params.missing_required_fields(),
            Self::RemoveDuplicates(params) => params.missing_required_fields(),
            Self::Filter(params) => params.missing_required_fields(),
            Self::Merge(params) => params.missing_required_fields(),
            Self::ReferenceIndex(params) => params.missing_required_fields(),
            Self::HostDepletion(params) => params.missing_required_fields(),
            Self::ReferenceContaminantDepletion(params) => params.missing_required_fields(),
            Self::Rrna(params) => params.missing_required_fields(),
            Self::Screen(params) => params.missing_required_fields(),
            Self::ReportQc(params) => params.missing_required_fields(),
            Self::OverrepresentedProfile(params) => params.missing_required_fields(),
            Self::PrimerNormalization(_)
            | Self::ChimeraDetection(_)
            | Self::AsvInference(_)
            | Self::OtuClustering(_)
            | Self::AbundanceNormalization(_) => Vec::new(),
        }
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        match self {
            Self::Validate(params) => params.retention_conditions(),
            Self::Stats(params) => params.retention_conditions(),
            Self::ReadLengthProfile(params) => serde_json::to_value(params).unwrap_or_default(),
            Self::EstimateLibraryComplexityPrealign(params) => params.retention_conditions(),
            Self::Correct(params) => params.retention_conditions(),
            Self::Umi(params) => params.retention_conditions(),
            Self::DetectAdapters(params) => params.retention_conditions(),
            Self::Trim(params) => params.retention_conditions(),
            Self::TrimTerminalDamage(params) => params.retention_conditions(),
            Self::TrimPolygTails(params) => params.retention_conditions(),
            Self::RemoveDuplicates(params) => params.retention_conditions(),
            Self::Filter(params) => params.retention_conditions(),
            Self::Merge(params) => params.retention_conditions(),
            Self::ReferenceIndex(params) => params.retention_conditions(),
            Self::HostDepletion(params) => params.retention_conditions(),
            Self::ReferenceContaminantDepletion(params) => params.retention_conditions(),
            Self::Rrna(params) => params.retention_conditions(),
            Self::Screen(params) => params.retention_conditions(),
            Self::ReportQc(params) => params.retention_conditions(),
            Self::OverrepresentedProfile(params) => {
                serde_json::to_value(params).unwrap_or_default()
            }
            Self::PrimerNormalization(params) => serde_json::to_value(params).unwrap_or_default(),
            Self::ChimeraDetection(params) => serde_json::to_value(params).unwrap_or_default(),
            Self::AsvInference(params) => serde_json::to_value(params).unwrap_or_default(),
            Self::OtuClustering(params) => serde_json::to_value(params).unwrap_or_default(),
            Self::AbundanceNormalization(params) => {
                serde_json::to_value(params).unwrap_or_default()
            }
        }
    }
}
