//! Owner: pipeline array assembly and publication from borrowed upstream observations.
use crate::artifacts::hpc_array::pipeline::render_hpc_pipeline_array_script;
use crate::repo::hpc_array::{relative, ArrayKind, Destinations};
use anyhow::{ensure, Result};
use bijux_dna_bench_model::{
    build_hpc_pipeline_array_report, BenchmarkLocalAllDomainSlurmSubmitJob,
    BenchmarkLocalHpcPipelineNodeArrayReport, BenchmarkLocalHpcScratchLayoutJob,
    BenchmarkLocalPipelineDagValidationReport, HpcPipelineArrayInputs,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
/// An admitted destination can be prepared before acquiring expensive upstream observations.
pub struct HpcPipelineArrayDestination {
    root: PathBuf,
    paths: Destinations,
}
impl HpcPipelineArrayDestination {
    /// Admit a normalized output below the governed benchmark run root without writing files.
    /// # Errors
    /// Rejects output escape, root directories and invalid file stems.
    pub fn prepare(repo_root: &Path, script: &Path) -> Result<Self> {
        Ok(Self {
            root: repo_root.to_path_buf(),
            paths: Destinations::prepare(repo_root, script, ArrayKind::Pipeline)?,
        })
    }
}
/// Selected-job observations are shared by the scratch and execution collectors.
pub struct HpcPipelineArrayRequest<'a> {
    pub repo_root: &'a Path,
    pub destination: HpcPipelineArrayDestination,
    pub jobs: &'a [BenchmarkLocalAllDomainSlurmSubmitJob],
    pub scratch: &'a [BenchmarkLocalHpcScratchLayoutJob],
    pub pipelines: &'a BTreeMap<String, BenchmarkLocalPipelineDagValidationReport>,
    pub pipeline_order: &'a [&'a str],
}
/// Return the admitted report without publishing it.
/// # Errors
/// Rejects missing or ambiguous coverage and invalid resource envelopes.
pub fn collect_hpc_pipeline_array(
    request: HpcPipelineArrayRequest<'_>,
) -> Result<BenchmarkLocalHpcPipelineNodeArrayReport> {
    build(&request)
}
/// Publish the deterministic script and JSON manifest.
/// # Errors
/// Returns assembly or publication failures. The two writes are not a filesystem transaction.
pub fn publish_hpc_pipeline_array(
    request: HpcPipelineArrayRequest<'_>,
) -> Result<BenchmarkLocalHpcPipelineNodeArrayReport> {
    let report = build(&request)?;
    let script = render_hpc_pipeline_array_script(request.repo_root, &report)?;
    request.destination.paths.publish(&script, &report)?;
    Ok(report)
}
/// Compare both persisted outputs with current admitted observations.
/// # Errors
/// Returns assembly, acquisition or script/manifest drift failures.
pub fn validate_hpc_pipeline_array(
    request: HpcPipelineArrayRequest<'_>,
) -> Result<BenchmarkLocalHpcPipelineNodeArrayReport> {
    let report = build(&request)?;
    let script = render_hpc_pipeline_array_script(request.repo_root, &report)?;
    request.destination.paths.validate(&script, &report, &report.script_path)?;
    Ok(report)
}
fn build(
    request: &HpcPipelineArrayRequest<'_>,
) -> Result<BenchmarkLocalHpcPipelineNodeArrayReport> {
    ensure!(
        request.repo_root == request.destination.root,
        "HPC pipeline array destination belongs to a different repository root"
    );
    let paths = &request.destination.paths;
    build_hpc_pipeline_array_report(HpcPipelineArrayInputs {
        jobs: request.jobs,
        scratch: request.scratch,
        pipelines: request.pipelines,
        pipeline_order: request.pipeline_order,
        script_path: relative(request.repo_root, &paths.script),
        manifest_path: relative(request.repo_root, &paths.manifest),
        wrapper_log_root: relative(request.repo_root, &paths.logs),
    })
}
