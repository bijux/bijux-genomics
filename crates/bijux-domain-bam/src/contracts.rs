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
    let stage = crate::BamStage::try_from(stage_id).ok()?;
    match stage {
        crate::BamStage::Filter | crate::BamStage::Markdup | crate::BamStage::Recalibration => {
            Some(BamStageContract {
                input: BamArtifactKind::Bam,
                output: BamArtifactKind::Bam,
                emits_bam: true,
                emits_report: true,
            })
        }
        _ => Some(BamStageContract {
            input: BamArtifactKind::Bam,
            output: BamArtifactKind::Report,
            emits_bam: false,
            emits_report: true,
        }),
    }
}
