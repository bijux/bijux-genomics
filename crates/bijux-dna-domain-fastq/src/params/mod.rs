//! Owner: bijux-dna-domain-fastq
//! Canonical effective parameters for FASTQ stages.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::stages::ids::{
    STAGE_ABUNDANCE_NORMALIZATION, STAGE_ASV_INFERENCE, STAGE_CHIMERA_DETECTION, STAGE_CORRECT,
    STAGE_DETECT_ADAPTERS, STAGE_FILTER, STAGE_LOW_COMPLEXITY, STAGE_MERGE, STAGE_OTU_CLUSTERING,
    STAGE_PREPROCESS, STAGE_PRIMER_NORMALIZATION, STAGE_QC_POST, STAGE_RRNA, STAGE_SCREEN,
    STAGE_STATS_NEUTRAL, STAGE_TRIM, STAGE_UMI, STAGE_VALIDATE_PRE,
};
use bijux_dna_core::ids::StageId;

pub mod correct;
pub mod defaults;
pub mod detect_adapters;
pub mod edna;
pub mod filter;
pub mod merge;
pub mod preprocess;
pub mod qc_post;
pub mod screen;
pub mod stats;
pub mod trim;
pub mod umi;
pub mod validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageParamDescriptor {
    pub param_type_id: &'static str,
    pub schema_version: &'static str,
}

#[must_use]
pub fn stage_param_descriptor(stage_id: &StageId) -> Option<StageParamDescriptor> {
    if stage_id == &STAGE_VALIDATE_PRE {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.validate",
            schema_version: "legacy.unversioned",
        });
    }
    if stage_id == &STAGE_STATS_NEUTRAL {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.stats",
            schema_version: stats::STATS_SCHEMA_VERSION,
        });
    }
    if stage_id == &STAGE_CORRECT {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.correct",
            schema_version: correct::CORRECT_SCHEMA_VERSION,
        });
    }
    if stage_id == &STAGE_UMI {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.umi",
            schema_version: umi::UMI_SCHEMA_VERSION,
        });
    }
    if stage_id == &STAGE_DETECT_ADAPTERS {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.detect_adapters",
            schema_version: "legacy.unversioned",
        });
    }
    if stage_id == &STAGE_TRIM {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.trim",
            schema_version: "legacy.unversioned",
        });
    }
    if stage_id == &STAGE_FILTER {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.filter",
            schema_version: "legacy.unversioned",
        });
    }
    if stage_id == &STAGE_MERGE {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.merge",
            schema_version: "legacy.unversioned",
        });
    }
    if stage_id == &STAGE_RRNA {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.rrna",
            schema_version: "legacy.unversioned",
        });
    }
    if stage_id == &STAGE_SCREEN {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.screen",
            schema_version: "legacy.unversioned",
        });
    }
    if stage_id == &STAGE_QC_POST {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.qc_post",
            schema_version: "legacy.unversioned",
        });
    }
    if stage_id == &STAGE_PREPROCESS {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.preprocess",
            schema_version: "legacy.unversioned",
        });
    }
    if stage_id == &STAGE_PRIMER_NORMALIZATION {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.primer_normalization",
            schema_version: edna::EDNA_SCHEMA_VERSION,
        });
    }
    if stage_id == &STAGE_CHIMERA_DETECTION {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.chimera_detection",
            schema_version: edna::EDNA_SCHEMA_VERSION,
        });
    }
    if stage_id == &STAGE_ASV_INFERENCE {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.asv_inference",
            schema_version: edna::EDNA_SCHEMA_VERSION,
        });
    }
    if stage_id == &STAGE_OTU_CLUSTERING {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.otu_clustering",
            schema_version: edna::EDNA_SCHEMA_VERSION,
        });
    }
    if stage_id == &STAGE_ABUNDANCE_NORMALIZATION {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.abundance_normalization",
            schema_version: edna::EDNA_SCHEMA_VERSION,
        });
    }
    None
}

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
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "stage")]
pub enum EffectiveParams {
    Validate(validate::ValidateEffectiveParams),
    Stats(stats::FastqStatsParams),
    Correct(correct::FastqCorrectParams),
    Umi(umi::FastqUmiParams),
    DetectAdapters(detect_adapters::DetectAdaptersEffectiveParams),
    Trim(trim::TrimEffectiveParams),
    Filter(filter::FilterEffectiveParams),
    Merge(merge::MergeEffectiveParams),
    Rrna(screen::RrnaEffectiveParams),
    Screen(screen::ScreenEffectiveParams),
    QcPost(qc_post::QcPostEffectiveParams),
    Preprocess(preprocess::PreprocessEffectiveParams),
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
            Self::Correct(params) => params.missing_required_fields(),
            Self::Umi(params) => params.missing_required_fields(),
            Self::DetectAdapters(params) => params.missing_required_fields(),
            Self::Trim(params) => params.missing_required_fields(),
            Self::Filter(params) => params.missing_required_fields(),
            Self::Merge(params) => params.missing_required_fields(),
            Self::Rrna(params) => params.missing_required_fields(),
            Self::Screen(params) => params.missing_required_fields(),
            Self::QcPost(params) => params.missing_required_fields(),
            Self::Preprocess(params) => params.missing_required_fields(),
            Self::PrimerNormalization(_) => Vec::new(),
            Self::ChimeraDetection(_) => Vec::new(),
            Self::AsvInference(_) => Vec::new(),
            Self::OtuClustering(_) => Vec::new(),
            Self::AbundanceNormalization(_) => Vec::new(),
        }
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        match self {
            Self::Validate(params) => params.retention_conditions(),
            Self::Stats(params) => params.retention_conditions(),
            Self::Correct(params) => params.retention_conditions(),
            Self::Umi(params) => params.retention_conditions(),
            Self::DetectAdapters(params) => params.retention_conditions(),
            Self::Trim(params) => params.retention_conditions(),
            Self::Filter(params) => params.retention_conditions(),
            Self::Merge(params) => params.retention_conditions(),
            Self::Rrna(params) => params.retention_conditions(),
            Self::Screen(params) => params.retention_conditions(),
            Self::QcPost(params) => params.retention_conditions(),
            Self::Preprocess(params) => params.retention_conditions(),
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

#[must_use]
pub fn parse_effective_params(
    stage_id: &StageId,
    value: &serde_json::Value,
) -> Option<EffectiveParams> {
    if stage_id == &STAGE_VALIDATE_PRE {
        return serde_json::from_value::<validate::ValidateEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Validate);
    }
    if stage_id == &STAGE_STATS_NEUTRAL {
        return serde_json::from_value::<stats::FastqStatsParams>(value.clone())
            .ok()
            .map(EffectiveParams::Stats);
    }
    if stage_id == &STAGE_CORRECT {
        return serde_json::from_value::<correct::FastqCorrectParams>(value.clone())
            .ok()
            .map(EffectiveParams::Correct);
    }
    if stage_id == &STAGE_UMI {
        return serde_json::from_value::<umi::FastqUmiParams>(value.clone())
            .ok()
            .map(EffectiveParams::Umi);
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
    if stage_id == &STAGE_LOW_COMPLEXITY {
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
        return serde_json::from_value::<screen::RrnaEffectiveParams>(value.clone())
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
    if stage_id == &STAGE_PRIMER_NORMALIZATION {
        return serde_json::from_value::<edna::PrimerNormalizationEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::PrimerNormalization);
    }
    if stage_id == &STAGE_CHIMERA_DETECTION {
        return serde_json::from_value::<edna::ChimeraDetectionEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::ChimeraDetection);
    }
    if stage_id == &STAGE_ASV_INFERENCE {
        return serde_json::from_value::<edna::AsvInferenceEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::AsvInference);
    }
    if stage_id == &STAGE_OTU_CLUSTERING {
        return serde_json::from_value::<edna::OtuClusteringEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::OtuClustering);
    }
    if stage_id == &STAGE_ABUNDANCE_NORMALIZATION {
        return serde_json::from_value::<edna::AbundanceNormalizationEffectiveParams>(
            value.clone(),
        )
        .ok()
        .map(EffectiveParams::AbundanceNormalization);
    }
    None
}
