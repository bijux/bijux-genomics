use std::path::Path;

use crate::types::FastqArtifactKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FastqStage {
    PrepareReference,
    ValidateReads,
    ProfileReadLengths,
    DetectAdapters,
    DamageAwarePretrim,
    PrimerNormalization,
    PolygTailing,
    Trim,
    Filter,
    ProfileReads,
    Rrna,
    Merge,
    Deduplicate,
    LowComplexity,
    HostDepletion,
    ContaminantScreen,
    Correct,
    Umi,
    ProfileOverrepresentedSequences,
    ReportQc,
    Screen,
    ChimeraDetection,
    AsvInference,
    OtuClustering,
    AbundanceNormalization,
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

fn stats_stage_io(stage: FastqStage) -> StageContract {
    StageContract {
        stage,
        io: StageIO {
            inputs: vec![
                FastqArtifactKind::SingleEnd,
                FastqArtifactKind::PairedEnd,
                FastqArtifactKind::Merged,
            ],
            outputs: vec![FastqArtifactKind::StatsOnly],
            optional_outputs: if matches!(stage, FastqStage::Screen) {
                Some(vec![FastqArtifactKind::TaxonomyMapping])
            } else {
                None
            },
        },
    }
}

fn paired_fastq_stage_io(stage: FastqStage) -> StageContract {
    StageContract {
        stage,
        io: StageIO {
            inputs: vec![FastqArtifactKind::SingleEnd, FastqArtifactKind::PairedEnd],
            outputs: vec![FastqArtifactKind::SingleEnd, FastqArtifactKind::PairedEnd],
            optional_outputs: Some(vec![FastqArtifactKind::StatsOnly]),
        },
    }
}

fn broad_fastq_inputs() -> Vec<FastqArtifactKind> {
    vec![
        FastqArtifactKind::SingleEnd,
        FastqArtifactKind::PairedEnd,
        FastqArtifactKind::Merged,
    ]
}

#[must_use]
pub fn canonical_contract_for_stage(stage: FastqStage) -> StageContract {
    match stage {
        FastqStage::PrepareReference => StageContract {
            stage,
            io: StageIO {
                inputs: vec![FastqArtifactKind::ReferenceFasta],
                outputs: vec![FastqArtifactKind::ReferenceIndex],
                optional_outputs: None,
            },
        },
        FastqStage::ValidateReads
        | FastqStage::ProfileReadLengths
        | FastqStage::DetectAdapters
        | FastqStage::ProfileReads
        | FastqStage::ProfileOverrepresentedSequences
        | FastqStage::ReportQc
        | FastqStage::Screen => stats_stage_io(stage),
        FastqStage::DamageAwarePretrim
        | FastqStage::PrimerNormalization
        | FastqStage::PolygTailing
        | FastqStage::Trim
        | FastqStage::Filter
        | FastqStage::Deduplicate
        | FastqStage::LowComplexity
        | FastqStage::HostDepletion
        | FastqStage::ContaminantScreen
        | FastqStage::Rrna => paired_fastq_stage_io(stage),
        FastqStage::Merge => StageContract {
            stage,
            io: StageIO {
                inputs: vec![FastqArtifactKind::PairedEnd],
                outputs: vec![FastqArtifactKind::Merged],
                optional_outputs: Some(vec![FastqArtifactKind::PairedEnd]),
            },
        },
        FastqStage::Correct | FastqStage::Umi => StageContract {
            stage,
            io: StageIO {
                inputs: vec![FastqArtifactKind::PairedEnd],
                outputs: vec![FastqArtifactKind::PairedEnd],
                optional_outputs: None,
            },
        },
        FastqStage::ChimeraDetection => StageContract {
            stage,
            io: StageIO {
                inputs: broad_fastq_inputs(),
                outputs: broad_fastq_inputs(),
                optional_outputs: Some(vec![FastqArtifactKind::StatsOnly]),
            },
        },
        FastqStage::AsvInference => StageContract {
            stage,
            io: StageIO {
                inputs: broad_fastq_inputs(),
                outputs: vec![FastqArtifactKind::AmpliconTable],
                optional_outputs: Some(vec![FastqArtifactKind::RepresentativeFasta]),
            },
        },
        FastqStage::OtuClustering => StageContract {
            stage,
            io: StageIO {
                inputs: broad_fastq_inputs(),
                outputs: vec![
                    FastqArtifactKind::AmpliconTable,
                    FastqArtifactKind::RepresentativeFasta,
                ],
                optional_outputs: Some(vec![FastqArtifactKind::StatsOnly]),
            },
        },
        FastqStage::AbundanceNormalization => StageContract {
            stage,
            io: StageIO {
                inputs: vec![FastqArtifactKind::AmpliconTable],
                outputs: vec![FastqArtifactKind::AmpliconTable],
                optional_outputs: Some(vec![FastqArtifactKind::TaxonomyMapping]),
            },
        },
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct FastqStageContract {
    pub input_kind: FastqArtifactKind,
    pub output_kind: FastqArtifactKind,
    pub accepted_input_kinds: &'static [FastqArtifactKind],
    pub possible_output_kinds: &'static [FastqArtifactKind],
    pub may_drop_reads: bool,
    pub must_preserve_pairing: bool,
    pub emits_fastq: bool,
    pub preserves: &'static [&'static str],
    pub may_drop: &'static [&'static str],
    pub retention_definition: &'static str,
    pub retention_units: &'static str,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum QcClass {
    Structural,
    Statistical,
}

#[must_use]
pub fn qc_class_for_stage(stage_id: &str) -> Option<QcClass> {
    match stage_id {
        "fastq.validate_reads" => Some(QcClass::Structural),
        "fastq.profile_read_lengths"
        | "fastq.detect_adapters"
        | "fastq.profile_reads"
        | "fastq.profile_overrepresented_sequences"
        | "fastq.report_qc"
        | "fastq.screen_taxonomy" => Some(QcClass::Statistical),
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
