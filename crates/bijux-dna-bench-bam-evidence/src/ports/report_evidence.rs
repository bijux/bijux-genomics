use crate::observations::{ExpectedResultBinding, ReportStage, StageAdmissions, StageReadiness};
use anyhow::Result;
use std::path::Path;
/// Canonical observations supplied by composition, without native execution dependencies.
pub trait ReportEvidence {
    /// Acquire stage inventory from its canonical authority.
    ///
    /// # Errors
    /// Returns source acquisition, validation, or publication failure.
    fn stage_inventory(&self, repo_root: &Path) -> Result<Vec<StageReadiness>>;
    /// Acquire stage admissions from its canonical authority.
    ///
    /// # Errors
    /// Returns source acquisition, validation, or publication failure.
    fn stage_admissions(&self, repo_root: &Path) -> Result<StageAdmissions>;
    /// Acquire report stage from its canonical authority.
    ///
    /// # Errors
    /// Returns source acquisition, validation, or publication failure.
    fn report_stage(&self, repo_root: &Path, stage_id: &str) -> Result<ReportStage>;
    /// Acquire expected results from its canonical authority.
    ///
    /// # Errors
    /// Returns source acquisition, validation, or publication failure.
    fn expected_results(&self, repo_root: &Path) -> Result<Vec<ExpectedResultBinding>>;
}
