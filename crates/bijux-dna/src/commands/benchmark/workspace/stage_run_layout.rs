use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::bench::{
    normalize_benchmark_workspace_layout_report, rollback_benchmark_workspace_layout_report,
    WorkspaceNormalizationReport,
};

use super::{load_benchmark_workspace_config, BenchmarkWorkspaceConfig};

pub(crate) fn benchmark_runtime_corpus_dir_name(
    workspace: &BenchmarkWorkspaceConfig,
    _corpus_id: &str,
) -> Result<String> {
    if let Some(dir_name) = workspace
        .remote
        .as_ref()
        .and_then(|row| row.corpus_root.as_deref())
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
    {
        return Ok(dir_name.to_string());
    }
    Err(anyhow!("benchmark config is missing workspace.remote.corpus_root"))
}

pub(crate) fn benchmark_stage_run_relative_root(
    workspace: &BenchmarkWorkspaceConfig,
    scope: &str,
    corpus_dir_name: &str,
    stage_id: &str,
) -> Result<PathBuf> {
    let template = workspace
        .layout
        .as_ref()
        .and_then(|row| row.stage_runs.as_ref())
        .and_then(|row| match scope {
            "remote" => row.remote_results_template.as_deref(),
            "local-cache" => row.local_cache_results_template.as_deref(),
            "local-archive" => row.local_archive_results_template.as_deref(),
            _ => None,
        })
        .ok_or_else(|| anyhow!("benchmark config is missing workspace.layout.stage_runs template for scope `{scope}`"))?;
    Ok(PathBuf::from(
        template.replace("{corpus_id}", corpus_dir_name).replace("{stage_id}", stage_id),
    ))
}

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
