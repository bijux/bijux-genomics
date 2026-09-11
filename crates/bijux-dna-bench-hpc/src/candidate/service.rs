use super::{inputs::CandidateInputs, selection::select_candidate_run_rows, CandidateContext};
use anyhow::Result;
use bijux_dna_bench_model::{
    validate_benchmark_local_hpc_candidate_run_manifest,
    BenchmarkLocalHpcCandidateRunManifest as LocalHpcCandidateRunManifest,
};
use std::path::Path;
/// Collect and validate a governed candidate report from fresh local evidence.
///
/// # Errors
/// Rejects output confinement, invalid input evidence, selection and output contract failures.
pub fn collect_hpc_candidate_run_manifest(
    context: &CandidateContext<'_>,
    output: &Path,
) -> Result<LocalHpcCandidateRunManifest> {
    let absolute_output =
        if output.is_absolute() { output.to_path_buf() } else { context.repo_root.join(output) };
    context.evidence.admit_output(context.repo_root, &absolute_output)?;
    let inputs = CandidateInputs::acquire(context)?;
    let report = select_candidate_run_rows(context, &absolute_output, &inputs)?;
    validate_benchmark_local_hpc_candidate_run_manifest(&report)?;
    Ok(report)
}
