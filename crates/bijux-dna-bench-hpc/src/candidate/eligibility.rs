use super::{
    declarations::{
        EXCLUSION_HAS_DEPENDENCIES, EXCLUSION_INPUT_FOOTPRINT_EXCEEDED,
        EXCLUSION_MISSING_EXPECTED_OUTPUTS, EXCLUSION_MISSING_KNOWN_ASSETS,
        EXCLUSION_MISSING_RESOURCE_HINT, EXCLUSION_NON_BENCHMARK_RESULT,
        EXCLUSION_RESOURCE_ENVELOPE_EXCEEDED, EXCLUSION_UNAVAILABLE_EXECUTION_RESOLUTION,
        EXCLUSION_UNKNOWN_EXECUTION_MODE, SMALL_CPUS_PER_TASK_MAX, SMALL_INPUT_FOOTPRINT_MAX_BYTES,
        SMALL_MEMORY_MB_MAX, SMALL_SCRATCH_GB_MAX, SMALL_WALLTIME_MINUTES_MAX,
    },
    inputs::CandidateInputs,
    paths::path_relative_to_repo,
    resources::{parse_time_limit_minutes, staged_input_bytes},
    CandidateContext,
};
use anyhow::Result;
use bijux_dna_bench_model::{
    BenchmarkLocalAllDomainSlurmSubmitJob as BenchLocalAllDomainSlurmSubmitJob,
    BenchmarkLocalHpcAssetStagingInput as LocalHpcAssetStagingInput,
    BenchmarkLocalHpcExecutionResolverRow as LocalHpcExecutionResolverRow,
    BenchmarkLocalHpcScratchJobResources as LocalHpcJobResourceHint, HpcJobResourceHints,
};
use std::collections::BTreeMap;
#[derive(Debug)]
pub(super) struct EligibleCandidateJob<'a> {
    pub representative_group_id: String,
    pub selection_reason: String,
    pub result_id: String,
    pub submit_job: &'a BenchLocalAllDomainSlurmSubmitJob,
    pub execution: &'a LocalHpcExecutionResolverRow,
    pub resource_hint: LocalHpcJobResourceHint,
    pub walltime_minutes: u32,
    pub stage_result_manifest_path: String,
    pub result_root: String,
    pub staged_inputs: Vec<LocalHpcAssetStagingInput>,
    pub staged_input_total_bytes: u64,
}
pub(super) struct EligibleCandidates<'a> {
    pub rows: Vec<EligibleCandidateJob<'a>>,
    pub benchmark_job_count: usize,
    pub root_benchmark_job_count: usize,
    pub exclusions: BTreeMap<String, usize>,
}
struct CandidateIndex<'a> {
    assets: BTreeMap<&'a str, &'a [LocalHpcAssetStagingInput]>,
    execution: BTreeMap<&'a str, &'a LocalHpcExecutionResolverRow>,
    resources: &'a HpcJobResourceHints,
}
pub(super) fn collect_eligible<'a>(
    context: &CandidateContext<'_>,
    inputs: &'a CandidateInputs,
) -> Result<EligibleCandidates<'a>> {
    let index = CandidateIndex {
        assets: inputs
            .assets
            .jobs
            .iter()
            .map(|job| (job.result_id.as_str(), job.staged_inputs.as_slice()))
            .collect(),
        execution: inputs.execution.rows.iter().map(|row| (row.tool_id.as_str(), row)).collect(),
        resources: &inputs.resources,
    };
    let mut selected = EligibleCandidates {
        rows: Vec::new(),
        benchmark_job_count: 0,
        root_benchmark_job_count: 0,
        exclusions: BTreeMap::new(),
    };
    for job in &inputs.jobs {
        let Some(result_id) = job.result_id.as_deref() else {
            increment_count(&mut selected.exclusions, EXCLUSION_NON_BENCHMARK_RESULT);
            continue;
        };
        selected.benchmark_job_count += 1;
        if !job.dependencies.is_empty() {
            increment_count(&mut selected.exclusions, EXCLUSION_HAS_DEPENDENCIES);
            continue;
        }
        selected.root_benchmark_job_count += 1;
        if let Some(row) =
            admit_root_job(context, job, result_id, &index, &mut selected.exclusions)?
        {
            selected.rows.push(row);
        }
    }
    Ok(selected)
}
fn admit_root_job<'a>(
    context: &CandidateContext<'_>,
    job: &'a BenchLocalAllDomainSlurmSubmitJob,
    result_id: &str,
    index: &CandidateIndex<'a>,
    exclusions: &mut BTreeMap<String, usize>,
) -> Result<Option<EligibleCandidateJob<'a>>> {
    if job.outputs.is_empty() {
        increment_count(exclusions, EXCLUSION_MISSING_EXPECTED_OUTPUTS);
        return Ok(None);
    }
    let Some(staged_inputs) = index.assets.get(result_id).filter(|inputs| !inputs.is_empty())
    else {
        increment_count(exclusions, EXCLUSION_MISSING_KNOWN_ASSETS);
        return Ok(None);
    };
    let Some(execution) = index
        .execution
        .get(job.tool_id.as_str())
        .filter(|row| row.execution_mode != "unclassified")
    else {
        increment_count(exclusions, EXCLUSION_UNKNOWN_EXECUTION_MODE);
        return Ok(None);
    };
    if execution.resolution_kind.contains("unavailable") {
        increment_count(exclusions, EXCLUSION_UNAVAILABLE_EXECUTION_RESOLUTION);
        return Ok(None);
    }
    let Some(resources) = admit_resources(job, staged_inputs, index.resources, exclusions)? else {
        return Ok(None);
    };
    let result_paths = context.evidence.result_paths(context.repo_root, job)?;
    Ok(Some(EligibleCandidateJob {
        representative_group_id: format!("{}:{}", job.domain, execution.execution_mode),
        selection_reason: format!(
            "smallest eligible dependency-free benchmark row for {} / {}",
            job.domain, execution.execution_mode
        ),
        result_id: result_id.to_string(),
        submit_job: job,
        execution,
        resource_hint: resources.hint,
        walltime_minutes: resources.walltime_minutes,
        stage_result_manifest_path: path_relative_to_repo(
            context.repo_root,
            &result_paths.stage_result_manifest_path,
        ),
        result_root: path_relative_to_repo(context.repo_root, &result_paths.result_root),
        staged_inputs: staged_inputs.to_vec(),
        staged_input_total_bytes: resources.input_bytes,
    }))
}
struct AdmittedResources {
    hint: LocalHpcJobResourceHint,
    walltime_minutes: u32,
    input_bytes: u64,
}
fn admit_resources(
    job: &BenchLocalAllDomainSlurmSubmitJob,
    inputs: &[LocalHpcAssetStagingInput],
    hints: &HpcJobResourceHints,
    exclusions: &mut BTreeMap<String, usize>,
) -> Result<Option<AdmittedResources>> {
    let Some(hint) = hints.get(&(job.domain.clone(), job.stage_id.clone(), job.tool_id.clone()))
    else {
        increment_count(exclusions, EXCLUSION_MISSING_RESOURCE_HINT);
        return Ok(None);
    };
    let walltime_minutes = parse_time_limit_minutes(&hint.time_limit)?;
    if hint.cpus_per_task > SMALL_CPUS_PER_TASK_MAX
        || hint.memory_mb > SMALL_MEMORY_MB_MAX
        || walltime_minutes > SMALL_WALLTIME_MINUTES_MAX
        || hint.scratch_gb > SMALL_SCRATCH_GB_MAX
    {
        increment_count(exclusions, EXCLUSION_RESOURCE_ENVELOPE_EXCEEDED);
        return Ok(None);
    }
    let input_bytes = staged_input_bytes(inputs)?;
    if input_bytes > SMALL_INPUT_FOOTPRINT_MAX_BYTES {
        increment_count(exclusions, EXCLUSION_INPUT_FOOTPRINT_EXCEEDED);
        return Ok(None);
    }
    Ok(Some(AdmittedResources { hint: hint.clone(), walltime_minutes, input_bytes }))
}
pub(super) fn increment_count(counts: &mut BTreeMap<String, usize>, key: &str) {
    *counts.entry(key.to_string()).or_default() += 1;
}
