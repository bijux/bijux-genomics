use crate::observations::OutputAssessment;
use anyhow::Result;
use std::path::Path;
/// Canonical observations supplied by composition, without native execution dependencies.
pub trait OutputEvidence {
    /// Acquire stage output ids from its canonical authority.
    ///
    /// # Errors
    /// Returns source acquisition, validation, or publication failure.
    fn stage_output_ids(&self, repo_root: &Path, stage_id: &str) -> Vec<String>;
    /// Acquire assess output from its canonical authority.
    ///
    /// # Errors
    /// Returns source acquisition, validation, or publication failure.
    fn assess_output(
        &self,
        repo_root: &Path,
        stage_id: &str,
        tool_id: &str,
    ) -> Result<OutputAssessment>;
}
