use super::model::{ReportQcUpstreamStage, REPORT_QC_UPSTREAM_STAGES};
use crate::campaign::paths::benchmark_runtime_env;
use crate::catalog::corpus_fastq_stage_catalog_entry as stage_command_spec;
use crate::ports::CampaignCommand;
use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_corpus::normalized::NormalizedFastqSample as CorpusNormalizedSample;
use std::fs;
pub(crate) fn ensure_report_qc_upstream_stage_outputs(
    context: &super::PreparationContext<'_>,
    sample: &CorpusNormalizedSample,
    stage_id: &str,
) -> Result<()> {
    let super::PreparationContext {
        program,
        repo_root,
        workspace_config,
        corpus_id,
        platform,
        dry_run,
        executor,
    } = *context;
    let upstream = report_qc_upstream_stage(stage_id)?;
    let stage_spec = stage_command_spec(upstream.stage_id)?;
    let out_root = workspace_config.stage_out_root(corpus_id, upstream.stage_id)?;
    fs::create_dir_all(&out_root).with_context(|| format!("create {}", out_root.display()))?;

    let mut command_args = vec![
        "--platform".to_string(),
        platform.to_string(),
        "bench".to_string(),
        "fastq".to_string(),
        stage_spec.bench_subcommand.to_string(),
        "--sample-id".to_string(),
        sample.sample_id.clone(),
        "--r1".to_string(),
        sample.r1.display().to_string(),
        "--out".to_string(),
        out_root.display().to_string(),
        "--tools".to_string(),
        upstream.tool_id.to_string(),
    ];
    if let Some(r2) = sample.r2.as_ref() {
        command_args.push("--r2".to_string());
        command_args.push(r2.display().to_string());
    }
    command_args.extend(upstream.extra_args.iter().copied().map(str::to_string));

    if dry_run {
        return Ok(());
    }

    let command_name =
        program.to_str().ok_or_else(|| anyhow!("benchmark executable path is not valid UTF-8"))?;
    let exit_code = executor
        .execute(CampaignCommand {
            program: command_name,
            arguments: &command_args,
            cwd: Some(repo_root),
            environment: Some(&benchmark_runtime_env(&out_root)),
        })
        .with_context(|| format!("run {}", command_args.join(" ")))?;
    if exit_code != 0 {
        return Err(anyhow!(
            "{} governed QC bootstrap failed for {} with exit code {}",
            upstream.stage_id,
            sample.sample_id,
            exit_code
        ));
    }
    Ok(())
}

pub(crate) fn report_qc_upstream_stage(stage_id: &str) -> Result<ReportQcUpstreamStage> {
    REPORT_QC_UPSTREAM_STAGES
        .iter()
        .copied()
        .find(|row| row.stage_id == stage_id)
        .ok_or_else(|| anyhow!("unsupported report-qc upstream stage `{stage_id}`"))
}
