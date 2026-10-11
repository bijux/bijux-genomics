//! Owner: runner bundle transport contracts.
use std::path::{Path, PathBuf};

pub use bijux_dna_bench_model::BundleSidecar;

#[derive(Debug, Clone)]
pub struct BundleWriteRequest<'a> {
    pub output_path: &'a Path,
    pub bundle_kind: &'a str,
    pub campaign_id: &'a str,
    pub domain: &'a str,
    pub stage: &'a str,
    pub tool: &'a str,
    pub sample: &'a str,
    pub planned_job_id: &'a str,
    pub scheduler_job_id: &'a str,
    pub submitted_at: &'a str,
    pub backend: &'a str,
    pub recipients: &'a [String],
    pub plaintext: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct BundleDecryptRequest<'a> {
    pub bundle_path: &'a Path,
    pub sidecar_path: Option<&'a Path>,
    pub identity_files: &'a [PathBuf],
}
