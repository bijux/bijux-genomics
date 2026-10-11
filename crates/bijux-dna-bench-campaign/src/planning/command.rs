use crate::catalog::CorpusFastqStageCatalogEntry as StageCommandSpec;
use bijux_dna_bench_corpus::normalized::NormalizedFastqSample as CorpusNormalizedSample;
use std::path::Path;
#[derive(Clone, Copy)]
pub(crate) struct StageCommandRequest<'a> {
    pub stage_spec: StageCommandSpec,
    pub platform: &'a str,
    pub sample: &'a CorpusNormalizedSample,
    pub out_root: &'a Path,
    pub tools: &'a [String],
    pub threads: u32,
    pub jobs: u32,
    pub stage_args: &'a [String],
    pub extra_stage_args: &'a [String],
}
pub(crate) fn build_stage_command_args(request: StageCommandRequest<'_>) -> Vec<String> {
    let StageCommandRequest {
        stage_spec,
        platform,
        sample,
        out_root,
        tools,
        threads,
        jobs,
        stage_args,
        extra_stage_args,
    } = request;
    let mut command = vec![
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
        tools.join(","),
        "--threads".to_string(),
        threads.to_string(),
    ];
    if let Some(r2) = sample.r2.as_ref() {
        command.push("--r2".to_string());
        command.push(r2.display().to_string());
    }
    if jobs > 1 {
        command.push("--jobs".to_string());
        command.push(jobs.to_string());
    }
    command.extend(stage_args.iter().cloned());
    command.extend(extra_stage_args.iter().cloned());
    command
}
