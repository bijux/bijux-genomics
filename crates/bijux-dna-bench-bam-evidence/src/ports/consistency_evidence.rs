use crate::observations::{BindingSurface, CommandSurfaces};
use anyhow::Result;
use std::path::Path;
/// Canonical observations supplied by composition, without native execution dependencies.
pub trait ConsistencyEvidence {
    /// Acquire publish active bindings from its canonical authority.
    ///
    /// # Errors
    /// Returns source acquisition, validation, or publication failure.
    fn publish_active_bindings(&self, repo_root: &Path) -> Result<BindingSurface>;
    /// Acquire publish commands from its canonical authority.
    ///
    /// # Errors
    /// Returns source acquisition, validation, or publication failure.
    fn publish_commands(&self, repo_root: &Path) -> Result<CommandSurfaces>;
    /// Acquire publish expected results from its canonical authority.
    ///
    /// # Errors
    /// Returns source acquisition, validation, or publication failure.
    fn publish_expected_results(&self, repo_root: &Path) -> Result<BindingSurface>;
}
