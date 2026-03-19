#![allow(dead_code)]

use bijux_dna_core::{contract::PipelineSpec, ids::StageId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageCriticality {
    Essential,
    Optional,
    Experimental,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum FastqPipelineMode {
    Shotgun,
    Amplicon,
}

impl FastqPipelineMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Shotgun => "shotgun",
            Self::Amplicon => "amplicon",
        }
    }
}

#[must_use]
pub fn canonical_stage_order() -> Vec<StageId> {
    vec![
        StageId::from_static("fastq.validate_reads"),
        StageId::from_static("fastq.detect_adapters"),
        StageId::from_static("fastq.damage_aware_pretrim"),
        StageId::from_static("fastq.trim_reads"),
        StageId::from_static("fastq.filter_reads"),
        StageId::from_static("fastq.profile_reads"),
    ]
}

#[must_use]
pub fn canonical_amplicon_stage_order() -> Vec<StageId> {
    vec![
        StageId::from_static("fastq.validate_reads"),
        StageId::from_static("fastq.detect_adapters"),
        StageId::from_static("fastq.damage_aware_pretrim"),
        StageId::from_static("fastq.primer_normalization"),
        StageId::from_static("fastq.trim_reads"),
        StageId::from_static("fastq.filter_reads"),
        StageId::from_static("fastq.chimera_detection"),
        StageId::from_static("fastq.asv_inference"),
        StageId::from_static("fastq.abundance_normalization"),
        StageId::from_static("fastq.profile_reads"),
    ]
}

#[must_use]
pub fn optional_branches() -> Vec<(StageId, Vec<StageId>)> {
    vec![
        (
            StageId::from_static("fastq.merge"),
            vec![
                StageId::from_static("fastq.trim_reads"),
                StageId::from_static("fastq.filter_reads"),
            ],
        ),
        (
            StageId::from_static("fastq.correct"),
            vec![StageId::from_static("fastq.trim_reads")],
        ),
        (
            StageId::from_static("fastq.umi"),
            vec![StageId::from_static("fastq.trim_reads")],
        ),
        (
            StageId::from_static("fastq.report_qc"),
            vec![StageId::from_static("fastq.validate_reads")],
        ),
        (
            StageId::from_static("fastq.screen_taxonomy"),
            vec![StageId::from_static("fastq.validate_reads")],
        ),
    ]
}

#[must_use]
pub fn forbidden_transitions() -> Vec<(StageId, StageId)> {
    vec![
        (
            StageId::from_static("fastq.validate_reads"),
            StageId::from_static("fastq.merge"),
        ),
        (
            StageId::from_static("fastq.profile_reads"),
            StageId::from_static("fastq.trim_reads"),
        ),
        (
            StageId::from_static("fastq.profile_reads"),
            StageId::from_static("fastq.filter_reads"),
        ),
        (
            StageId::from_static("fastq.profile_reads"),
            StageId::from_static("fastq.merge"),
        ),
        (
            StageId::from_static("fastq.asv_inference"),
            StageId::from_static("fastq.otu_clustering"),
        ),
        (
            StageId::from_static("fastq.otu_clustering"),
            StageId::from_static("fastq.asv_inference"),
        ),
    ]
}

#[must_use]
pub fn stage_criticality(stage_id: &StageId) -> Option<StageCriticality> {
    match stage_id.as_str() {
        "fastq.validate_reads"
        | "fastq.detect_adapters"
        | "fastq.damage_aware_pretrim"
        | "fastq.trim_reads"
        | "fastq.merge"
        | "fastq.correct"
        | "fastq.filter_reads"
        | "fastq.profile_reads"
        | "fastq.primer_normalization"
        | "fastq.chimera_detection"
        | "fastq.abundance_normalization" => Some(StageCriticality::Essential),
        "fastq.asv_inference"
        | "fastq.otu_clustering"
        | "fastq.report_qc"
        | "fastq.umi"
        => Some(StageCriticality::Optional),
        "fastq.screen_taxonomy" => Some(StageCriticality::Experimental),
        _ => None,
    }
}

#[must_use]
pub fn preprocess_pipeline() -> PipelineSpec {
    PipelineSpec {
        stages: canonical_stage_order()
            .into_iter()
            .map(|stage| stage.as_str().to_string())
            .collect(),
    }
}

#[must_use]
pub fn preprocess_pipeline_for_mode(mode: FastqPipelineMode) -> PipelineSpec {
    let stages = match mode {
        FastqPipelineMode::Shotgun => canonical_stage_order(),
        FastqPipelineMode::Amplicon => canonical_amplicon_stage_order(),
    };
    PipelineSpec {
        stages: stages
            .into_iter()
            .map(|stage| stage.as_str().to_string())
            .collect(),
    }
}
