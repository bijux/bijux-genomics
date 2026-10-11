use crate::observations::ServingInventory;
use anyhow::Result;
use std::path::Path;
/// Canonical observations supplied by composition, without native execution dependencies.
pub trait ServingEvidence {
    /// Acquire serving from its canonical authority.
    ///
    /// # Errors
    /// Returns source acquisition, validation, or publication failure.
    fn serving(&self, repo_root: &Path) -> Result<ServingInventory>;
    /// Acquire publish serving from its canonical authority.
    ///
    /// # Errors
    /// Returns source acquisition, validation, or publication failure.
    fn publish_serving(&self, repo_root: &Path) -> Result<ServingInventory>;
}
