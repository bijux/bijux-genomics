//! Owner: bijux-dna-bench-population
//! Ibd execution exclusion.
use crate::ibd::model::VCF_BENCHMARK_MUTATOR_LOCK_PATH;
use anyhow::anyhow;
use anyhow::Result;
use std::path::Path;

pub(crate) fn benchmark_output_lock(repo_root: &Path) -> Result<bijux_dna_infra::FileLock> {
    bijux_dna_infra::FileLock::try_acquire(&repo_root.join(VCF_BENCHMARK_MUTATOR_LOCK_PATH))
        .map_err(|err| anyhow!("acquire VCF benchmark output lock: {err}"))
}
