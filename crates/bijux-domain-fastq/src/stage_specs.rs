use std::path::Path;

use crate::types::FastqArtifactKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StageIO {
    pub inputs: Vec<FastqArtifactKind>,
    pub outputs: Vec<FastqArtifactKind>,
    pub optional_outputs: Option<Vec<FastqArtifactKind>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct FastqStageContract {
    pub input_kind: FastqArtifactKind,
    pub output_kind: FastqArtifactKind,
    pub may_drop_reads: bool,
    pub must_preserve_pairing: bool,
    pub emits_fastq: bool,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum QcClass {
    Structural,
    Statistical,
}

#[must_use]
pub fn qc_class_for_stage(stage_id: &str) -> Option<QcClass> {
    match stage_id {
        "fastq.validate_pre" => Some(QcClass::Structural),
        "fastq.detect_adapters" | "fastq.qc_post" => Some(QcClass::Statistical),
        _ => None,
    }
}

#[must_use]
pub fn infer_input_kind(r2: Option<&Path>) -> FastqArtifactKind {
    if r2.is_some() {
        FastqArtifactKind::PairedEnd
    } else {
        FastqArtifactKind::SingleEnd
    }
}
