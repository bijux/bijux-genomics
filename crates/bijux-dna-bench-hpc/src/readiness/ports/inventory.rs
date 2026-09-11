use super::ReadinessDomain;
use anyhow::Result;
use bijux_dna_bench_model::BenchmarkLocalStageInventory;
use std::path::{Path, PathBuf};
/// Supply typed local inventory evidence; provider failures remain explicit readiness diagnostics.
pub trait StageEvidence {
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn load_local_stage_inventory(
        &self,
        repo_root: &Path,
        domain: ReadinessDomain,
    ) -> Result<BenchmarkLocalStageInventory>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn materialize_local_stage(&self, repo_root: &Path, stage_id: &str) -> Result<PathBuf>;
}
