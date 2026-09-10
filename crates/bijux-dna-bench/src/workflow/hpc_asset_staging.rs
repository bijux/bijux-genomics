//! Owner: staging collection over explicit command rows and stage hints.
use crate::repo::hpc_asset_staging::{observe_jobs, prepare, prepare_directories, relative};
use crate::{
    AllDomainRenderedCommandRow, BenchmarkLocalHpcAssetStagingManifest, HpcStageInputHint,
};
use anyhow::Result;
use std::{collections::BTreeMap, path::Path};
/// Current collection inputs independent of CLI or planner types.
pub struct HpcAssetStagingRequest<'a> {
    pub repository_root: &'a Path,
    pub output_path: &'a Path,
    pub staging_root: &'a Path,
    pub command_rows: Vec<AllDomainRenderedCommandRow>,
    pub stage_input_hints: &'a BTreeMap<String, Vec<HpcStageInputHint>>,
    pub generated_output_root: &'a str,
}
/// Observe current sources and derive the admitted staging manifest.
/// # Errors
/// Returns destination, input observation or manifest admission failures.
pub fn collect_hpc_asset_staging_manifest(
    request: HpcAssetStagingRequest<'_>,
) -> Result<BenchmarkLocalHpcAssetStagingManifest> {
    let root = request.repository_root;
    let (output, staging) = prepare(root, request.output_path, request.staging_root)?;
    prepare_directories(&output, &staging)?;
    let jobs = observe_jobs(
        root,
        &staging,
        request.command_rows,
        request.stage_input_hints,
        request.generated_output_root,
    )?;
    bijux_dna_bench_model::assemble_hpc_asset_staging_manifest(
        relative(root, &output),
        relative(root, &staging),
        jobs,
    )
}
