use crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig;
use anyhow::Result;
use bijux_dna_bench_campaign::ports::{
    CampaignClock, CampaignCommand, CampaignExecution, CampaignPaths,
};
use std::path::{Path, PathBuf};
pub(super) struct ConfiguredPaths<'a>(pub(super) &'a BenchmarkWorkspaceConfig);
impl CampaignPaths for ConfiguredPaths<'_> {
    fn stage_out_root(&self, corpus_id: &str, stage_id: &str) -> Result<PathBuf> {
        super::default_stage_out_root(self.0, corpus_id, stage_id)
    }
    fn apptainer_bind_root(&self) -> Option<PathBuf> {
        self.0
            .remote
            .as_ref()
            .and_then(|row| row.repo_root.as_ref())
            .and_then(|row| Path::new(row).parent())
            .map(Path::to_path_buf)
    }
}
pub(super) struct NativeExecution;
impl CampaignExecution for NativeExecution {
    fn execute(&self, command: CampaignCommand<'_>) -> Result<i32> {
        Ok(bijux_dna_api::v1::api::run::run_command_with_context(
            command.program,
            command.arguments,
            command.cwd,
            command.environment,
        )?
        .exit_code)
    }
}
pub(super) struct PublicationClock;
impl CampaignClock for PublicationClock {
    fn now(&self) -> std::time::SystemTime {
        std::time::SystemTime::now()
    }
}
