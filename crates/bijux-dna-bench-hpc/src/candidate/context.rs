use super::ports::CandidateEvidenceProvider;
use std::path::Path;
/// Explicit local composition without root discovery or global environment mutation.
pub struct CandidateContext<'a> {
    pub repo_root: &'a Path,
    pub evidence: &'a dyn CandidateEvidenceProvider,
}
