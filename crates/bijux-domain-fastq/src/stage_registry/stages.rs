use std::path::Path;

use crate::types::FastqArtifactKind;

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
