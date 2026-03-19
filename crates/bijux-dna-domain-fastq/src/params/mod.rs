//! Owner: bijux-dna-domain-fastq
//! Canonical effective parameters for FASTQ stages.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::stages::ids::{
    STAGE_NORMALIZE_ABUNDANCE, STAGE_INFER_ASVS, STAGE_REMOVE_CHIMERAS, STAGE_CORRECT_ERRORS,
    STAGE_TRIM_TERMINAL_DAMAGE, STAGE_DETECT_ADAPTERS, STAGE_FILTER_READS, STAGE_FILTER_LOW_COMPLEXITY,
    STAGE_MERGE_PAIRS, STAGE_CLUSTER_OTUS, STAGE_NORMALIZE_PRIMERS, STAGE_REPORT_QC,
    STAGE_DEPLETE_RRNA, STAGE_SCREEN_TAXONOMY, STAGE_PROFILE_READS, STAGE_TRIM_POLYG_TAILS,
    STAGE_TRIM_READS, STAGE_EXTRACT_UMIS, STAGE_VALIDATE_READS,
};
use bijux_dna_core::ids::StageId;

#[path = "processing/correct.rs"]
pub mod correct;
pub mod defaults;
#[path = "quality/detect_adapters.rs"]
pub mod detect_adapters;
#[path = "edna/edna.rs"]
pub mod edna;
#[path = "quality/filter.rs"]
pub mod filter;
#[path = "processing/merge.rs"]
pub mod merge;
#[path = "processing/preprocess.rs"]
pub mod preprocess;
#[path = "quality/qc_post.rs"]
pub mod qc_post;
#[path = "quality/screen.rs"]
pub mod screen;
#[path = "quality/stats.rs"]
pub mod stats;
#[path = "quality/trim.rs"]
pub mod trim;
#[path = "processing/umi.rs"]
pub mod umi;
#[path = "quality/validate.rs"]
pub mod validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageParamDescriptor {
    pub param_type_id: &'static str,
    pub schema_version: &'static str,
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn stage_param_descriptor(stage_id: &StageId) -> Option<StageParamDescriptor> {
    if stage_id == &STAGE_VALIDATE_READS {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.validate_reads",
            schema_version: "bijux.fastq.params.validate_reads.v1",
        });
    }
    if stage_id == &STAGE_PROFILE_READS {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.profile_reads",
            schema_version: stats::STATS_SCHEMA_VERSION,
        });
    }
    if stage_id == &STAGE_CORRECT_ERRORS {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.correct_errors",
            schema_version: correct::CORRECT_SCHEMA_VERSION,
        });
    }
    if stage_id == &STAGE_EXTRACT_UMIS {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.extract_umis",
            schema_version: umi::UMI_SCHEMA_VERSION,
        });
    }
    if stage_id == &STAGE_DETECT_ADAPTERS {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.detect_adapters",
            schema_version: "bijux.fastq.params.detect_adapters.v1",
        });
    }
    if stage_id == &STAGE_TRIM_READS {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.trim_reads",
            schema_version: "bijux.fastq.params.trim_reads.v1",
        });
    }
    if stage_id == &STAGE_TRIM_TERMINAL_DAMAGE {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.trim_terminal_damage",
            schema_version: "bijux.fastq.params.trim_terminal_damage.v1",
        });
    }
    if stage_id == &STAGE_TRIM_POLYG_TAILS {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.trim_polyg_tails",
            schema_version: "bijux.fastq.params.trim_polyg_tails.v1",
        });
    }
    if stage_id == &STAGE_FILTER_READS {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.filter_reads",
            schema_version: "bijux.fastq.params.filter_reads.v1",
        });
    }
    if stage_id == &STAGE_FILTER_LOW_COMPLEXITY {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.filter_low_complexity",
            schema_version: "bijux.fastq.params.filter_low_complexity.v1",
        });
    }
    if stage_id == &STAGE_MERGE_PAIRS {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.merge_pairs",
            schema_version: "bijux.fastq.params.merge_pairs.v1",
        });
    }
    if stage_id == &STAGE_DEPLETE_RRNA {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.deplete_rrna",
            schema_version: "bijux.fastq.params.deplete_rrna.v1",
        });
    }
    if stage_id == &STAGE_SCREEN_TAXONOMY {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.screen_taxonomy",
            schema_version: "bijux.fastq.params.screen_taxonomy.v1",
        });
    }
    if stage_id == &STAGE_REPORT_QC {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.report_qc",
            schema_version: "bijux.fastq.params.report_qc.v1",
        });
    }
    if stage_id == &STAGE_NORMALIZE_PRIMERS {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.normalize_primers",
            schema_version: edna::EDNA_SCHEMA_VERSION,
        });
    }
    if stage_id == &STAGE_REMOVE_CHIMERAS {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.remove_chimeras",
            schema_version: edna::EDNA_SCHEMA_VERSION,
        });
    }
    if stage_id == &STAGE_INFER_ASVS {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.infer_asvs",
            schema_version: edna::EDNA_SCHEMA_VERSION,
        });
    }
    if stage_id == &STAGE_CLUSTER_OTUS {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.cluster_otus",
            schema_version: edna::EDNA_SCHEMA_VERSION,
        });
    }
    if stage_id == &STAGE_NORMALIZE_ABUNDANCE {
        return Some(StageParamDescriptor {
            param_type_id: "fastq.normalize_abundance",
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
    ReportQc(qc_post::QcPostEffectiveParams),
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
            Self::ReportQc(params) => params.missing_required_fields(),
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
            Self::Correct(params) => params.retention_conditions(),
            Self::Umi(params) => params.retention_conditions(),
            Self::DetectAdapters(params) => params.retention_conditions(),
            Self::Trim(params) => params.retention_conditions(),
            Self::Filter(params) => params.retention_conditions(),
            Self::Merge(params) => params.retention_conditions(),
            Self::Rrna(params) => params.retention_conditions(),
            Self::Screen(params) => params.retention_conditions(),
            Self::ReportQc(params) => params.retention_conditions(),
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
    if stage_id == &STAGE_VALIDATE_READS {
        return serde_json::from_value::<validate::ValidateEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Validate);
    }
    if stage_id == &STAGE_PROFILE_READS {
        return serde_json::from_value::<stats::FastqStatsParams>(value.clone())
            .ok()
            .map(EffectiveParams::Stats);
    }
    if stage_id == &STAGE_CORRECT_ERRORS {
        return serde_json::from_value::<correct::FastqCorrectParams>(value.clone())
            .ok()
            .map(EffectiveParams::Correct);
    }
    if stage_id == &STAGE_EXTRACT_UMIS {
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
    if stage_id == &STAGE_TRIM_READS {
        return serde_json::from_value::<trim::TrimEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Trim);
    }
    if stage_id == &STAGE_TRIM_TERMINAL_DAMAGE {
        return serde_json::from_value::<trim::TrimEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Trim);
    }
    if stage_id == &STAGE_TRIM_POLYG_TAILS {
        return serde_json::from_value::<trim::TrimEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Trim);
    }
    if stage_id == &STAGE_FILTER_READS {
        return serde_json::from_value::<filter::FilterEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Filter);
    }
    if stage_id == &STAGE_FILTER_LOW_COMPLEXITY {
        return serde_json::from_value::<filter::FilterEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Filter);
    }
    if stage_id == &STAGE_MERGE_PAIRS {
        return serde_json::from_value::<merge::MergeEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Merge);
    }
    if stage_id == &STAGE_DEPLETE_RRNA {
        return serde_json::from_value::<screen::RrnaEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Rrna);
    }
    if stage_id == &STAGE_SCREEN_TAXONOMY {
        return serde_json::from_value::<screen::ScreenEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Screen);
    }
    if stage_id == &STAGE_REPORT_QC {
        return serde_json::from_value::<qc_post::QcPostEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::ReportQc);
    }
    if stage_id == &STAGE_NORMALIZE_PRIMERS {
        return serde_json::from_value::<edna::PrimerNormalizationEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::PrimerNormalization);
    }
    if stage_id == &STAGE_REMOVE_CHIMERAS {
        return serde_json::from_value::<edna::ChimeraDetectionEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::ChimeraDetection);
    }
    if stage_id == &STAGE_INFER_ASVS {
        return serde_json::from_value::<edna::AsvInferenceEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::AsvInference);
    }
    if stage_id == &STAGE_CLUSTER_OTUS {
        return serde_json::from_value::<edna::OtuClusteringEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::OtuClustering);
    }
    if stage_id == &STAGE_NORMALIZE_ABUNDANCE {
        return serde_json::from_value::<edna::AbundanceNormalizationEffectiveParams>(
            value.clone(),
        )
        .ok()
        .map(EffectiveParams::AbundanceNormalization);
    }
    None
}
