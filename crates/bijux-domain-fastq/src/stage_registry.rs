//! FASTQ stage registry and contracts.

mod contract;
mod semantics;
mod stages;

use serde::{Deserialize, Serialize};

use crate::types::FastqArtifactKind;

pub use bijux_core::RawFailure;
pub use contract::{
    assess_merge_suitability, contract_for_stage, ensure_umi_headers, inspect_headers,
    log_header_warnings, normalize_outputs, preflight_stage, HeaderInspection, MergeSuitability,
    NormalizedOutputs,
};
pub use semantics::{
    fastq_stage_is_stable, stage_criticality, stage_kind, stage_metric_classes,
    stage_metric_invariants, stage_semantics, BoundaryInvariant, FastqStageKind, StageDefinition,
    StageSemantics, STAGES, STAGE_BOUNDARY_INVARIANTS,
};
pub use stages::{infer_input_kind, qc_class_for_stage, FastqStageContract, QcClass};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FastqStage {
    Preprocess,
    ValidatePre,
    Trim,
    Filter,
    Merge,
    Correct,
    StatsNeutral,
    QcPost,
    Umi,
    Screen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageIO {
    pub inputs: Vec<FastqArtifactKind>,
    pub outputs: Vec<FastqArtifactKind>,
    pub optional_outputs: Option<Vec<FastqArtifactKind>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageContract {
    pub stage: FastqStage,
    pub io: StageIO,
}

#[must_use]
pub fn canonical_contract_for_stage(stage: FastqStage) -> StageContract {
    match stage {
        FastqStage::Preprocess => StageContract {
            stage,
            io: StageIO {
                inputs: vec![FastqArtifactKind::SingleEnd, FastqArtifactKind::PairedEnd],
                outputs: vec![FastqArtifactKind::StatsOnly],
                optional_outputs: None,
            },
        },
        FastqStage::ValidatePre
        | FastqStage::Trim
        | FastqStage::Filter
        | FastqStage::Correct
        | FastqStage::Umi => StageContract {
            stage,
            io: StageIO {
                inputs: vec![FastqArtifactKind::SingleEnd, FastqArtifactKind::PairedEnd],
                outputs: vec![FastqArtifactKind::SingleEnd, FastqArtifactKind::PairedEnd],
                optional_outputs: None,
            },
        },
        FastqStage::Merge => StageContract {
            stage,
            io: StageIO {
                inputs: vec![FastqArtifactKind::PairedEnd],
                outputs: vec![FastqArtifactKind::Merged],
                optional_outputs: Some(vec![FastqArtifactKind::PairedEnd]),
            },
        },
        FastqStage::StatsNeutral | FastqStage::QcPost | FastqStage::Screen => StageContract {
            stage,
            io: StageIO {
                inputs: vec![
                    FastqArtifactKind::SingleEnd,
                    FastqArtifactKind::PairedEnd,
                    FastqArtifactKind::Merged,
                ],
                outputs: vec![FastqArtifactKind::StatsOnly],
                optional_outputs: None,
            },
        },
    }
}
