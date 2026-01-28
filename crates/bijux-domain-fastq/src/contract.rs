//! Canonical FASTQ pipeline contract.

use serde::{Deserialize, Serialize};

use crate::contracts::FastqArtifactKind;

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
pub fn contract_for_stage(stage: FastqStage) -> StageContract {
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
