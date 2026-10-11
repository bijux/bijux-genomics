use super::model::{CampaignScope, PlannedSamples};
use crate::ports::CampaignClock;
use anyhow::{Context, Result};
use bijux_dna_bench_corpus::run::{
    validate_benchmark_local_corpus_run, BenchmarkCorpusRunManifest,
    BENCHMARK_LOCAL_CORPUS_RUN_SCHEMA_V1,
};
use std::fs;
pub(super) fn publish_run(
    scope: &CampaignScope<'_>,
    planned: PlannedSamples,
    clock: &dyn CampaignClock,
) -> Result<BenchmarkCorpusRunManifest> {
    let args = scope.args;
    let contract = &args.contract;
    let corpus_spec = &args.source;
    let corpus_root = &args.corpus_root;
    let repo_root = args.repo_root.clone();
    let out_root = args.out_root.clone();
    let platform = args.platform.clone();
    let tools = scope.tools.clone();
    let resolved_stage_args = scope.resolved_stage_args.clone();
    let workspace_config_path = &args.config_path;
    let publication_config_path = &args.publication_config_path;
    let PlannedSamples { mut runs, extra_manifest_fields, .. } = planned;
    runs.sort_by(|left, right| left.sample_id.cmp(&right.sample_id));
    let failures = runs.iter().filter(|row| row.exit_code != 0).count();
    let manifest = BenchmarkCorpusRunManifest {
        schema_version: BENCHMARK_LOCAL_CORPUS_RUN_SCHEMA_V1.to_string(),
        generated_at_utc: bijux_dna_bench_corpus::run::format_generated_at_utc(clock.now())?,
        corpus_id: corpus_spec.corpus_id.clone(),
        stage_id: contract.stage_id.clone(),
        scenario_id: contract.scenario_id.clone(),
        sample_scope: contract.sample_scope.clone(),
        tool_kind: "benchmark".to_string(),
        platform,
        tools,
        threads: args.threads,
        jobs: args.jobs,
        sample_jobs: args.sample_jobs.max(1),
        sample_limit: (args.sample_limit > 0).then_some(args.sample_limit),
        dry_run: args.dry_run,
        config_path: workspace_config_path.display().to_string(),
        publication_config_path: publication_config_path.display().to_string(),
        repo_root: repo_root.display().to_string(),
        corpus_root: corpus_root.display().to_string(),
        out_root: out_root.display().to_string(),
        stage_args: resolved_stage_args.clone(),
        samples_total: runs.len(),
        samples_failed: failures,
        runs,
        extra_fields: extra_manifest_fields,
    };
    validate_benchmark_local_corpus_run(&manifest)?;
    let manifest_path = out_root.join("run_manifest.json");
    fs::write(&manifest_path, format!("{}\n", serde_json::to_string_pretty(&manifest)?))
        .with_context(|| format!("write {}", manifest_path.display()))?;

    Ok(manifest)
}
