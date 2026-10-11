use super::admission::{resolve_tools, validate_benchmark_layout};
use super::execution::execute_samples;
use super::model::CampaignScope;
use super::paths::benchmark_runtime_env;
use super::publication::publish_run;
use super::samples::plan_samples;
use crate::cache::resolve_deplete_rrna_stage_options;
use crate::catalog::corpus_fastq_stage_catalog_entry as stage_command_spec;
use crate::planning::{collect_stage_manifest_fields, merge_stage_args_from_config};
use crate::ports::{CampaignClock, CampaignExecution, CampaignPaths};
use crate::request::CorpusCampaignRequest;
use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_corpus::normalized::{
    discover_normalized_fastq_samples as discover_normalized_samples,
    match_normalized_fastq_corpus_metadata as validate_corpus_contract,
    normalized_fastq_expected_sample_total as corpus_expected_sample_total,
    select_normalized_fastq_paired_samples as select_paired_samples,
};
use bijux_dna_bench_corpus::run::BenchmarkCorpusRunManifest;
use std::fs;
/// Execute and publish one corpus campaign using explicit dependencies.
///
/// # Errors
/// Reject invalid source/selection/layout or propagate preparation, transport and publication failures.
/// Failed tool exits are retained in the returned manifest for caller presentation and exit mapping.
pub fn run_corpus_campaign(
    args: &CorpusCampaignRequest,
    workspace_config: &dyn CampaignPaths,
    executor: &dyn CampaignExecution,
    clock: &dyn CampaignClock,
) -> Result<BenchmarkCorpusRunManifest> {
    let corpus_spec = &args.source;
    let contract = &args.contract;
    let stage_spec = stage_command_spec(&contract.stage_id)?;
    let corpus_root = args.corpus_root.clone();
    let out_root = args.out_root.clone();
    if corpus_spec.corpus_id != args.corpus_id {
        return Err(anyhow!(
            "configured corpus spec drift: expected `{}`, found `{}`",
            args.corpus_id,
            corpus_spec.corpus_id
        ));
    }

    validate_benchmark_layout(&corpus_root, &out_root)?;

    let all_samples = discover_normalized_samples(
        &corpus_root,
        &corpus_spec.corpus_id,
        corpus_expected_sample_total(corpus_spec),
    )?;
    let metadata_by_sample = validate_corpus_contract(&corpus_root, corpus_spec, &all_samples)?;
    let mut selected_samples = match contract.sample_scope.as_str() {
        "full" => all_samples,
        "paired" => select_paired_samples(corpus_spec, &all_samples, &metadata_by_sample)?,
        other => {
            return Err(anyhow!(
                "unsupported corpus benchmark sample scope `{other}` for {}",
                contract.stage_id
            ));
        }
    };
    if args.sample_limit > 0 && args.sample_limit < selected_samples.len() {
        selected_samples.truncate(args.sample_limit);
    }

    let tools = resolve_tools(&contract.tools, &args.tools)?;
    let runtime_env = benchmark_runtime_env(&out_root);
    let (resolved_stage_args, resolved_manifest_args) =
        merge_stage_args_from_config(&args.defaults, &args.stage_args, &args.manifest_args)?;
    let extra_manifest_fields = collect_stage_manifest_fields(
        &contract.stage_id,
        &resolved_stage_args,
        &resolved_manifest_args,
    )?;
    let deplete_rrna_options = if contract.stage_id == "fastq.deplete_rrna" {
        Some(resolve_deplete_rrna_stage_options(&resolved_stage_args)?)
    } else {
        None
    };

    if args.threads == 0 || args.jobs == 0 {
        return Err(anyhow!("FASTQ corpus benchmark resources must be non-zero"));
    }
    if args.platform.trim().is_empty() {
        return Err(anyhow!("benchmark platform must be a non-empty identifier"));
    }
    for sample in &selected_samples {
        crate::admission::path_component(&sample.sample_id, "sample identifier")?;
    }
    fs::create_dir_all(&out_root).with_context(|| format!("create {}", out_root.display()))?;
    let scope = CampaignScope {
        args,
        workspace_config,
        executor,
        stage_spec,
        tools,
        resolved_stage_args,
        deplete_rrna_options,
        runtime_env,
    };
    let planned = plan_samples(&scope, selected_samples, extra_manifest_fields)?;
    let executed = execute_samples(&scope, planned)?;
    publish_run(&scope, executed, clock)
}
