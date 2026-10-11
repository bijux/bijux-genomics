use super::observations::{
    ExpectedResultBinding, OutputAssessment, OutputDeclaration, RenderedCommandObservation,
    ServingBinding, StageAdmission, StageInventory,
};
use anyhow::Result;
use std::{collections::BTreeMap, path::Path};
/// Acquire canonical source evidence without frontend dependencies or hidden fallback.
pub trait OutputEvidence {
    /// Acquire serving evidence.
    ///
    /// # Errors
    /// Propagates canonical declaration or acquisition failure.
    fn serving(&self, root: &Path) -> Result<Vec<ServingBinding>>;
    /// Acquire assess evidence.
    ///
    /// # Errors
    /// Propagates canonical declaration or acquisition failure.
    fn assess(
        &self,
        root: &Path,
        stage_id: &str,
        tool_id: &str,
    ) -> Result<(OutputDeclaration, OutputAssessment)>;
}
/// Acquire canonical source evidence without frontend dependencies or hidden fallback.
pub trait ReportEvidence {
    /// Acquire expected evidence.
    ///
    /// # Errors
    /// Propagates canonical declaration or acquisition failure.
    fn expected(&self, root: &Path) -> Result<Vec<ExpectedResultBinding>>;
    /// Acquire inventory evidence.
    ///
    /// # Errors
    /// Propagates canonical declaration or acquisition failure.
    fn inventory(&self, root: &Path) -> Result<StageInventory>;
    /// Acquire admissions evidence.
    ///
    /// # Errors
    /// Propagates canonical declaration or acquisition failure.
    fn admissions(&self, root: &Path) -> Result<BTreeMap<String, Vec<StageAdmission>>>;
}
/// Acquire canonical source evidence without frontend dependencies or hidden fallback.
pub trait CommandEvidence {
    /// Acquire active evidence.
    ///
    /// # Errors
    /// Propagates canonical declaration or acquisition failure.
    fn active(
        &self,
        root: &Path,
    ) -> Result<crate::readiness::active::FastqActiveStageToolMatrixCollection>;
    /// Acquire rendered evidence.
    ///
    /// # Errors
    /// Propagates canonical declaration or acquisition failure.
    fn rendered(&self, root: &Path) -> Result<Vec<RenderedCommandObservation>>;
}
