#![allow(dead_code)]

use bijux_core::{contract::PipelineSpec, ids::StageId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageCriticality {
    Essential,
    Optional,
    Experimental,
}

#[must_use]
pub fn canonical_stage_order() -> Vec<StageId> {
    vec![
        StageId::from_static("fastq.validate_pre"),
        StageId::from_static("fastq.detect_adapters"),
        StageId::from_static("fastq.trim"),
        StageId::from_static("fastq.filter"),
        StageId::from_static("fastq.stats_neutral"),
    ]
}

#[must_use]
pub fn optional_branches() -> Vec<(StageId, Vec<StageId>)> {
    vec![
        (
            StageId::from_static("fastq.merge"),
            vec![
                StageId::from_static("fastq.trim"),
                StageId::from_static("fastq.filter"),
            ],
        ),
        (
            StageId::from_static("fastq.correct"),
            vec![StageId::from_static("fastq.trim")],
        ),
        (
            StageId::from_static("fastq.umi"),
            vec![StageId::from_static("fastq.trim")],
        ),
        (
            StageId::from_static("fastq.qc_post"),
            vec![StageId::from_static("fastq.validate_pre")],
        ),
        (
            StageId::from_static("fastq.screen"),
            vec![StageId::from_static("fastq.validate_pre")],
        ),
    ]
}

#[must_use]
pub fn forbidden_transitions() -> Vec<(StageId, StageId)> {
    vec![
        (
            StageId::from_static("fastq.validate_pre"),
            StageId::from_static("fastq.merge"),
        ),
        (
            StageId::from_static("fastq.stats_neutral"),
            StageId::from_static("fastq.trim"),
        ),
        (
            StageId::from_static("fastq.stats_neutral"),
            StageId::from_static("fastq.filter"),
        ),
        (
            StageId::from_static("fastq.stats_neutral"),
            StageId::from_static("fastq.merge"),
        ),
    ]
}

#[must_use]
pub fn stage_criticality(stage_id: &StageId) -> Option<StageCriticality> {
    match stage_id.as_str() {
        "fastq.validate_pre"
        | "fastq.detect_adapters"
        | "fastq.trim"
        | "fastq.merge"
        | "fastq.correct"
        | "fastq.filter"
        | "fastq.stats_neutral" => Some(StageCriticality::Essential),
        "fastq.qc_post" | "fastq.umi" | "fastq.preprocess" => Some(StageCriticality::Optional),
        "fastq.screen" => Some(StageCriticality::Experimental),
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
