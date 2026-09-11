use crate::observations::{
    BamStageAssessment, BamToolAssessment, CorpusDeclaration, Domain, FastqAssessment,
    StageInventory,
};
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Path;
/// Explicit canonical acquisition boundary; implementations own their documented local effects.
pub trait ServingEvidence {
    /// Acquire canonical corpus declarations.
    ///
    /// # Errors
    /// Returns the underlying source or declaration error.
    fn corpus_declarations(&self, repo_root: &Path) -> Result<Vec<CorpusDeclaration>>;
    /// Acquire canonical integrated tools.
    ///
    /// # Errors
    /// Returns the underlying source or declaration error.
    fn integrated_tools(&self, repo_root: &Path) -> Result<BTreeSet<String>>;
    /// Acquire canonical stage inventory.
    ///
    /// # Errors
    /// Returns the underlying source or declaration error.
    fn stage_inventory(&self, repo_root: &Path, domain: Domain) -> Result<StageInventory>;
    /// Acquire canonical fastq tools.
    ///
    /// # Errors
    /// Returns the underlying source or declaration error.
    fn fastq_tools(&self, repo_root: &Path, stage_id: &str) -> Result<Vec<String>>;
    /// Acquire canonical fastq assessment.
    ///
    /// # Errors
    /// Returns the underlying source or declaration error.
    fn fastq_assessment(
        &self,
        repo_root: &Path,
        stage_id: &str,
        tool_id: &str,
    ) -> Result<FastqAssessment>;
    /// Acquire canonical bam stage.
    ///
    /// # Errors
    /// Returns the underlying source or declaration error.
    fn bam_stage(&self, repo_root: &Path, stage_id: &str) -> Result<BamStageAssessment>;
    /// Acquire canonical bam assessment.
    ///
    /// # Errors
    /// Returns the underlying source or declaration error.
    fn bam_assessment(
        &self,
        repo_root: &Path,
        stage_id: &str,
        tool_id: &str,
    ) -> Result<BamToolAssessment>;
}
