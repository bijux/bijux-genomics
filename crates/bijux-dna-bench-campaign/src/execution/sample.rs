use super::{PendingSampleRun, PostSuccessAction};
use crate::cache::{promote_sortmerna_sample_index_cache, prune_sortmerna_sample_payload};
use crate::ports::{CampaignCommand, CampaignExecution};
use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_corpus::run::BenchmarkCorpusSampleRun;
use std::path::Path;
pub(crate) fn execute_sample(
    program: &Path,
    repo_root: &Path,
    row: PendingSampleRun,
    executor: &dyn CampaignExecution,
) -> Result<BenchmarkCorpusSampleRun> {
    let command_name =
        program.to_str().ok_or_else(|| anyhow!("benchmark executable path is not valid UTF-8"))?;
    let exit_code = executor
        .execute(CampaignCommand {
            program: command_name,
            arguments: &row.command_args,
            cwd: Some(repo_root),
            environment: Some(&row.env_bindings),
        })
        .with_context(|| format!("run {}", row.command.join(" ")))?;
    if exit_code == 0 {
        if let Some(action) = row.post_success_action.as_ref() {
            match action {
                PostSuccessAction::PromoteAndPruneSortmernaCache {
                    out_root,
                    sample_id,
                    rrna_bundle_id,
                } => {
                    promote_sortmerna_sample_index_cache(out_root, sample_id, rrna_bundle_id)?;
                    prune_sortmerna_sample_payload(out_root, sample_id)?;
                }
            }
        }
    }
    Ok(BenchmarkCorpusSampleRun {
        sample_id: row.sample.sample_id,
        r1: row.sample.r1.display().to_string(),
        r2: row.sample.r2.map(|path| path.display().to_string()),
        layout: row.sample.layout,
        status: if exit_code == 0 { "completed".to_string() } else { "failed".to_string() },
        exit_code,
        command: row.command,
        report_json: row.report_json.display().to_string(),
        extra_fields: row.extra_fields,
    })
}
