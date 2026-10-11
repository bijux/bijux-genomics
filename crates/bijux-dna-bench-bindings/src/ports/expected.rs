use crate::observations::{CorpusAssignment, Domain, OutputContract};
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
/// Explicit canonical acquisition boundary; implementations own their documented local effects.
pub trait ExpectedEvidence {
    /// Acquire canonical fixture sample scopes.
    ///
    /// # Errors
    /// Returns the underlying source or declaration error.
    fn fixture_sample_scopes(&self, repo_root: &Path) -> Result<BTreeMap<String, String>>;
    /// Acquire canonical output contracts.
    ///
    /// # Errors
    /// Returns the underlying source or declaration error.
    fn output_contracts(&self, repo_root: &Path, domain: Domain) -> Result<Vec<OutputContract>>;
    /// Acquire canonical corpus assignments.
    ///
    /// # Errors
    /// Returns the underlying source or declaration error.
    fn corpus_assignments(&self, repo_root: &Path, domain: Domain)
        -> Result<Vec<CorpusAssignment>>;
    /// Acquire canonical readiness kinds.
    ///
    /// # Errors
    /// Returns the underlying source or declaration error.
    fn readiness_kinds(&self, repo_root: &Path, domain: Domain)
        -> Result<BTreeMap<String, String>>;
}
