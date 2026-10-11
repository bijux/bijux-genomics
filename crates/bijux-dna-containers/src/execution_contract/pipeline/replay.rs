use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{ensure, Context, Result};

use crate::site_policy::ContainerGovernance;

use super::admission::{
    admit_container_pipeline_site, ContainerPipelineSiteAdmission,
    ContainerPipelineSiteAdmissionRequest,
};
use super::model::ContainerPipelineContract;
use super::validation::load_container_pipeline;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricalContainerPipelineReplayRequest {
    pub governance: ContainerGovernance,
    pub hostname: String,
    pub source_pipeline_path: PathBuf,
    pub replay_pipeline_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricalContainerPipelineReplayAdmission {
    pub source_pipeline_path: PathBuf,
    pub source_pipeline_sha256: String,
    pub replay_pipeline_path: PathBuf,
    pub replay_pipeline_sha256: String,
    pub site: ContainerPipelineSiteAdmission,
}

/// Admit an exact legacy protocol replay after repository and image currentization.
///
/// The source and replay declarations must both use the legacy schema. Every scientific field is
/// required to remain identical; only the frozen repository and immutable image identities may
/// differ. The replay declaration then undergoes normal site admission.
///
/// # Errors
/// Returns an error when either declaration is unsafe or malformed, scientific intent changed,
/// or the replay declaration fails host and filesystem admission.
pub fn admit_historical_container_pipeline_replay(
    request: &HistoricalContainerPipelineReplayRequest,
) -> Result<HistoricalContainerPipelineReplayAdmission> {
    let source_pipeline_path =
        exact_regular_file(&request.source_pipeline_path, "source pipeline")?;
    let replay_pipeline_path =
        exact_regular_file(&request.replay_pipeline_path, "replay pipeline")?;
    ensure!(
        source_pipeline_path != replay_pipeline_path,
        "historical replay requires distinct source and replay declarations"
    );

    let source = legacy_pipeline(&source_pipeline_path, "source")?;
    let replay = legacy_pipeline(&replay_pipeline_path, "replay")?;
    let mut expected = source.clone();
    expected.repository = replay.repository.clone();
    expected.image = replay.image.clone();
    ensure!(
        expected == replay,
        "historical replay changed fields outside repository and image identity"
    );

    let site = admit_container_pipeline_site(&ContainerPipelineSiteAdmissionRequest {
        governance: request.governance.clone(),
        hostname: request.hostname.clone(),
        pipeline_path: replay_pipeline_path.clone(),
    })?;
    Ok(HistoricalContainerPipelineReplayAdmission {
        source_pipeline_sha256: bijux_dna_infra::hash_file_sha256(&source_pipeline_path)?,
        replay_pipeline_sha256: bijux_dna_infra::hash_file_sha256(&replay_pipeline_path)?,
        source_pipeline_path,
        replay_pipeline_path,
        site,
    })
}

fn legacy_pipeline(path: &Path, label: &str) -> Result<Box<super::model::ContainerPipelineV1>> {
    match load_container_pipeline(path)? {
        ContainerPipelineContract::Legacy(pipeline) => Ok(pipeline),
        ContainerPipelineContract::Current(_) => {
            anyhow::bail!("historical replay {label} declaration must use the legacy schema")
        }
    }
}

fn exact_regular_file(path: &Path, label: &str) -> Result<PathBuf> {
    ensure!(path.is_absolute(), "{label} path must be absolute");
    let metadata = fs::symlink_metadata(path).with_context(|| format!("inspect {label}"))?;
    ensure!(
        !metadata.file_type().is_symlink() && metadata.is_file(),
        "{label} is not a regular file"
    );
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    ensure!(canonical == path, "{label} path is not canonical");
    Ok(canonical)
}
