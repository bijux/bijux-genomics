use crate::catalog::ProbeKind;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Published proof address and the observed producing tool identity.
#[derive(Debug, Clone)]
pub struct MaterializedProof {
    pub proof_path: PathBuf,
    pub observed_tool_id: String,
}

/// Canonical producer composition; scientific generation remains with each domain owner.
pub trait ProbeMaterializer {
    /// # Errors
    /// Propagate stage materialization failures with their original diagnostics.
    fn local_stage(&self, stage_id: &str) -> Result<PathBuf>;
    /// # Errors
    /// Reject stages without a governed tool declaration.
    fn stage_tool(&self, stage_id: &str) -> Result<String>;
    /// # Errors
    /// Reject unsupported producers or propagate generation failures.
    fn generate(&self, kind: ProbeKind, probe_root: &Path) -> Result<MaterializedProof>;
}
