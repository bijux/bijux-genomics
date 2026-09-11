use super::LocalReadinessAdapter;
use anyhow::Result;
use bijux_dna_api::v1::api::bench::BenchmarkLocalStageInventory;
use bijux_dna_bench_hpc::readiness::ports::{ReadinessDomain, StageEvidence};
use std::path::{Path, PathBuf};
impl StageEvidence for LocalReadinessAdapter {
    fn load_local_stage_inventory(
        &self,
        repo_root: &Path,
        domain: ReadinessDomain,
    ) -> Result<BenchmarkLocalStageInventory> {
        crate::commands::benchmark::local_stage_inventory::load_local_stage_inventory(
            repo_root,
            super::inventory_domain(domain),
        )
    }
    fn materialize_local_stage(&self, repo_root: &Path, stage_id: &str) -> Result<PathBuf> {
        crate::commands::benchmark::local_stage_commands::materialize_local_stage(
            repo_root, stage_id,
        )
    }
}
