//! Owner: stage array assembly and publication from borrowed upstream observations.
use crate::artifacts::hpc_stage_array::render_stage_benchmark_array_script;
use crate::repo::hpc_stage_array::{relative, Destinations};
use anyhow::{ensure, Result};
use bijux_dna_bench_model::{
    build_hpc_stage_array_report, BenchmarkLocalAllDomainSlurmSubmitJob,
    BenchmarkLocalHpcExecutionResolverRow, BenchmarkLocalHpcScratchLayoutJob,
    BenchmarkLocalHpcStageBenchmarkArrayReport, HpcStageArrayInputs,
};
use std::path::{Path, PathBuf};
/// An admitted destination can be prepared before acquiring expensive upstream observations.
pub struct HpcStageArrayDestination {
    root: PathBuf,
    paths: Destinations,
}
impl HpcStageArrayDestination {
    /// Admit a normalized output below the governed benchmark run root without writing files.
    /// # Errors
    /// Rejects output escape, root directories and invalid file stems.
    pub fn prepare(repo_root: &Path, script: &Path) -> Result<Self> {
        Ok(Self { root: repo_root.to_path_buf(), paths: Destinations::prepare(repo_root, script)? })
    }
}
/// Selected-job observations are shared by the scratch and execution collectors.
pub struct HpcStageArrayRequest<'a> {
    pub repo_root: &'a Path,
    pub destination: HpcStageArrayDestination,
    pub jobs: &'a [BenchmarkLocalAllDomainSlurmSubmitJob],
    pub scratch: &'a [BenchmarkLocalHpcScratchLayoutJob],
    pub executions: &'a [BenchmarkLocalHpcExecutionResolverRow],
}
/// Return the admitted report without publishing it.
/// # Errors
/// Rejects missing or ambiguous coverage and invalid resource envelopes.
pub fn collect_hpc_stage_array(
    request: HpcStageArrayRequest<'_>,
) -> Result<BenchmarkLocalHpcStageBenchmarkArrayReport> {
    build(&request)
}
/// Publish the deterministic script and JSON manifest.
/// # Errors
/// Returns assembly or publication failures. The two writes are not a filesystem transaction.
pub fn publish_hpc_stage_array(
    request: HpcStageArrayRequest<'_>,
) -> Result<BenchmarkLocalHpcStageBenchmarkArrayReport> {
    let report = build(&request)?;
    let script = render_stage_benchmark_array_script(request.repo_root, &report)?;
    request.destination.paths.publish(&script, &report)?;
    Ok(report)
}
/// Compare both persisted outputs with current admitted observations.
/// # Errors
/// Returns assembly, acquisition or script/manifest drift failures.
pub fn validate_hpc_stage_array(
    request: HpcStageArrayRequest<'_>,
) -> Result<BenchmarkLocalHpcStageBenchmarkArrayReport> {
    let report = build(&request)?;
    let script = render_stage_benchmark_array_script(request.repo_root, &report)?;
    request.destination.paths.validate(&script, &report)?;
    Ok(report)
}
fn build(request: &HpcStageArrayRequest<'_>) -> Result<BenchmarkLocalHpcStageBenchmarkArrayReport> {
    ensure!(
        request.repo_root == request.destination.root,
        "HPC stage array destination belongs to a different repository root"
    );
    let paths = &request.destination.paths;
    build_hpc_stage_array_report(HpcStageArrayInputs {
        jobs: request.jobs,
        scratch: request.scratch,
        executions: request.executions,
        script_path: relative(request.repo_root, &paths.script),
        manifest_path: relative(request.repo_root, &paths.manifest),
        wrapper_log_root: relative(request.repo_root, &paths.logs),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stage_array_destination_rejects_escape_and_foreign_repository_before_writes() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path();
        for path in
            ["outside.sbatch", "artifacts/runs/bench", "artifacts/runs/bench/../escape.sbatch"]
        {
            assert!(HpcStageArrayDestination::prepare(root, Path::new(path)).is_err());
        }
        let destination = HpcStageArrayDestination::prepare(
            root,
            Path::new("artifacts/runs/bench/array.sbatch"),
        )?;
        let other = root.join("other");
        let error = publish_hpc_stage_array(HpcStageArrayRequest {
            repo_root: &other,
            destination,
            jobs: &[],
            scratch: &[],
            executions: &[],
        })
        .err()
        .ok_or_else(|| anyhow::anyhow!("foreign root admitted"))?;
        assert!(error.to_string().contains("different repository root"));
        assert!(!root.join("artifacts").exists());
        Ok(())
    }
}
