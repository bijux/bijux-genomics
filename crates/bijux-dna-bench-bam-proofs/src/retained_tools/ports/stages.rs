use anyhow::Result;
use std::path::Path;
/// An exact scientific proof capability, never an implicit substitute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageProofKind {
    OverlapCorrection,
    EndogenousContent,
    Damage,
    Authenticity,
    Contamination,
    Sex,
    Haplogroups,
    Recalibration,
    Genotyping,
    Kinship,
}
/// Cardinalities from the canonical stage-completion report.
#[derive(Debug, Clone, Copy)]
pub struct StageCompletion {
    pub complete_row_count: usize,
    pub active_row_count: usize,
    pub checked_surface_count: usize,
}
/// Compose existing scientific proof services without frontend dependencies.
pub trait StageProofs {
    /// Assess and publish the specified scientific proof.
    ///
    /// # Errors
    /// Returns the original feature, provider, scientific assessment or publication error.
    fn assess(&self, root: &Path, kind: StageProofKind, output: &Path) -> Result<StageCompletion>;
}
