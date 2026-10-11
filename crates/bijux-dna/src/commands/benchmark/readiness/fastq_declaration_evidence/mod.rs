use bijux_dna_bench_fastq::readiness::declarations::{
    CommandContext, OutputContext, ReportContext,
};
use std::path::Path;
mod commands;
mod outputs;
mod reports;
pub(super) fn outputs(root: &Path) -> OutputContext<'_> {
    OutputContext {
        repo_root: root,
        evidence: &outputs::OutputAdapter,
        run_id: crate::commands::benchmark::local_slurm_run_paths::LOCAL_SLURM_DRY_RUN_RUN_ID,
    }
}
pub(super) fn reports(root: &Path) -> ReportContext<'_> {
    ReportContext { repo_root: root, evidence: &reports::ReportAdapter }
}
pub(super) fn commands(root: &Path) -> CommandContext<'_> {
    CommandContext { repo_root: root, evidence: &commands::CommandAdapter }
}
