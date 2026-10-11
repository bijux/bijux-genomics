//! Owner: bounded current submission-manifest and script observations.
use super::slurm_scripts::{self, InspectionPaths};
use anyhow::{ensure, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkLocalSlurmDependencyCheckReport, BenchmarkLocalSlurmSubmitManifest,
};
use std::path::{Path, PathBuf};

pub(crate) struct DependencyPaths {
    pub(crate) scripts: InspectionPaths,
    pub(crate) manifest: PathBuf,
}

pub(crate) fn prepare(
    repo: &Path,
    root: &Path,
    manifest: &Path,
    report: &Path,
) -> Result<DependencyPaths> {
    let scripts = slurm_scripts::prepare(repo, root, report)?;
    let input = slurm_scripts::prepare(repo, root, manifest)?;
    Ok(DependencyPaths { scripts, manifest: input.report })
}

pub(crate) fn read_manifest(paths: &DependencyPaths) -> Result<BenchmarkLocalSlurmSubmitManifest> {
    super::admit_existing_path(&paths.scripts.repository, &paths.manifest)?;
    let bytes = bijux_dna_infra::read_to_end_bounded(&paths.manifest, 16 * 1024 * 1024)
        .context("read Slurm dependency input manifest")?;
    serde_json::from_slice(&bytes).context("parse Slurm dependency input manifest")
}

pub(crate) fn read_script(paths: &DependencyPaths, requested: &str) -> Result<(String, String)> {
    let request = Path::new(requested);
    let path = if request.is_absolute() {
        request.to_path_buf()
    } else {
        let from_repository = paths.scripts.repository.join(request);
        // Preserve the supported root-relative form only when no repository entry exists.
        match std::fs::symlink_metadata(&from_repository) {
            Ok(_) => from_repository,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                paths.scripts.root.join(request)
            }
            Err(error) => return Err(error).context("inspect Slurm script location"),
        }
    };
    ensure!(
        path.starts_with(&paths.scripts.root),
        "dependency script is outside the declared inventory root"
    );
    ensure!(
        path.extension().is_some_and(|extension| extension == "sbatch"),
        "dependency script must be an sbatch artifact"
    );
    super::admit_existing_path(&paths.scripts.repository, &path)?;
    let body = slurm_scripts::read_body(&path)?;
    Ok((paths.scripts.display(&path), body))
}

pub(crate) fn publish(
    paths: &DependencyPaths,
    report: &BenchmarkLocalSlurmDependencyCheckReport,
) -> Result<()> {
    super::admit_existing_path(&paths.scripts.repository, &paths.scripts.report)?;
    bijux_dna_infra::atomic_write_json(&paths.scripts.report, report)
        .context("publish Slurm dependency diagnostic")
}
