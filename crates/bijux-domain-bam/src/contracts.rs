//! BAM stage contracts and artifacts.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BamArtifactKind {
    Bam,
    BamIndex,
    ReferenceFasta,
    ReferenceIndex,
    ReferenceDict,
    BedRegions,
    Report,
}

#[derive(Debug, Clone, Copy)]
pub struct BamStageContract {
    pub input: BamArtifactKind,
    pub output: BamArtifactKind,
    pub emits_bam: bool,
    pub emits_report: bool,
}

#[must_use]
pub fn contract_for_stage(stage_id: &str) -> Option<BamStageContract> {
    match stage_id {
        "bam.filter" | "bam.markdup" | "bam.recalibration" => Some(BamStageContract {
            input: BamArtifactKind::Bam,
            output: BamArtifactKind::Bam,
            emits_bam: true,
            emits_report: true,
        }),
        "bam.validate"
        | "bam.qc_pre"
        | "bam.complexity"
        | "bam.coverage"
        | "bam.damage"
        | "bam.authenticity"
        | "bam.contamination"
        | "bam.sex"
        | "bam.bias_mitigation"
        | "bam.haplogroups"
        | "bam.genotyping"
        | "bam.kinship" => Some(BamStageContract {
            input: BamArtifactKind::Bam,
            output: BamArtifactKind::Report,
            emits_bam: false,
            emits_report: true,
        }),
        _ => None,
    }
}
