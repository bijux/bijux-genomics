use crate::ports::{NativeProofWriter, ReadinessEvidence};
use std::path::Path;
/// Explicit repository and independently supplied declaration/execution boundaries.
pub struct ProofContext<'a> {
    pub repo_root: &'a Path,
    pub readiness: &'a dyn ReadinessEvidence,
    pub native: &'a dyn NativeProofWriter,
}
