use std::path::Path;

use anyhow::{anyhow, Context, Result};
pub(crate) use bijux_dna_api::v1::api::bench::{
    benchmark_runtime_corpus_dir_name, benchmark_stage_run_relative_root,
};
use bijux_dna_api::v1::api::bench::{
    normalize_benchmark_workspace_layout_report, rollback_benchmark_workspace_layout_report,
    WorkspaceNormalizationReport,
};

use super::load_benchmark_workspace_config;

pub(crate) fn run_normalize_workspace_layout(
    cwd: &Path,
    args: &crate::commands::cli::BenchNormalizeWorkspaceLayoutArgs,
) -> Result<()> {
    if let Some(report_path) = &args.rollback_report {
        let report: WorkspaceNormalizationReport = serde_json::from_slice(
            &std::fs::read(report_path)
                .with_context(|| format!("read {}", report_path.display()))?,
        )
        .with_context(|| format!("parse {}", report_path.display()))?;
        if report.corpus_id != args.corpus_id {
            return Err(anyhow!(
                "rollback report corpus `{}` does not match requested corpus `{}`",
                report.corpus_id,
                args.corpus_id
            ));
        }
        let restored = rollback_benchmark_workspace_layout_report(&report)?;
        println!("{}", serde_json::to_string_pretty(&restored)?);
        return Ok(());
    }
    let workspace = load_benchmark_workspace_config(cwd, args.config.as_deref())?;
    let corpus_dir_name = benchmark_runtime_corpus_dir_name(&workspace, &args.corpus_id)?;
    let report = normalize_benchmark_workspace_layout_report(
        &workspace,
        &args.corpus_id,
        &corpus_dir_name,
        args.confirm,
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
