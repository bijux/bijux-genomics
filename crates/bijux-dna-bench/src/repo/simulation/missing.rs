//! Owner: illustrative missing-result fixture paths, bounded observations and report publication.
use super::{admit_destination, paths::admit_entry, result_root};
use anyhow::{ensure, Context, Result};
use bijux_dna_bench_model::{
    all_domain_simulation_output_ids, all_domain_simulation_output_path,
    AllDomainMissingResultTestReport, AllDomainSimulationInput, BenchmarkStageResultManifest,
    DEFAULT_ALL_DOMAIN_MISSING_RESULT_FIXTURE_ROOT,
};
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

pub(crate) struct MissingResultPaths {
    pub(crate) repository: PathBuf,
    pub(crate) root: PathBuf,
    pub(crate) report: PathBuf,
}
impl MissingResultPaths {
    pub(crate) fn display(&self, path: &Path) -> Result<String> {
        path.strip_prefix(&self.repository)?
            .to_str()
            .map(str::to_owned)
            .context("missing-result evidence path must be UTF-8")
    }
}

pub(crate) fn prepare(
    repo: &Path,
    output: &Path,
    inputs: &[AllDomainSimulationInput<'_>],
) -> Result<MissingResultPaths> {
    let repository = repo.canonicalize().context("resolve missing-result repository")?;
    let root =
        admit_destination(&repository, Path::new(DEFAULT_ALL_DOMAIN_MISSING_RESULT_FIXTURE_ROOT))?;
    let report = if output.is_absolute() { output.to_path_buf() } else { repository.join(output) };
    let paths = MissingResultPaths { repository, root, report };
    ensure!(
        !paths.report.starts_with(&paths.root),
        "missing-result report must be outside its simulation fixture"
    );
    admit_entry(&paths.repository, &paths.report, false)?;
    paths.display(&paths.report)?;
    admit_entry(&paths.repository, &paths.root.join("manifest.json"), false)?;
    for input in inputs {
        let destination = result_root(&paths.root, input.expected);
        for name in ["command.sh", "stdout.txt", "stderr.txt", "metrics.json", "stage-result.json"]
        {
            admit_entry(&paths.repository, &destination.join(name), false)?;
        }
        for id in all_domain_simulation_output_ids(input.outputs) {
            let output =
                destination.join("declared-outputs").join(all_domain_simulation_output_path(&id));
            let directory = id.ends_with("_bundle") || id.ends_with("_dir");
            admit_entry(&paths.repository, &output, directory)?;
            if directory {
                admit_entry(
                    &paths.repository,
                    &output.join(".bijux-all-domain-fake-run-placeholder"),
                    false,
                )?;
            }
        }
    }
    Ok(paths)
}

pub(crate) fn remove_manifest(paths: &MissingResultPaths, requested: &str) -> Result<()> {
    let path = admit_destination(&paths.repository, Path::new(requested))?;
    ensure!(path.starts_with(&paths.root), "missing-result removal escapes its fixture");
    admit_entry(&paths.repository, &path, false)?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("remove {}", path.display())),
    }
}

pub(crate) fn read_manifest(
    paths: &MissingResultPaths,
    requested: &str,
) -> Result<Option<BenchmarkStageResultManifest>> {
    let path = admit_destination(&paths.repository, Path::new(requested))?;
    ensure!(path.starts_with(&paths.root), "missing-result observation escapes its fixture");
    admit_entry(&paths.repository, &path, false)?;
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("inspect missing-result manifest"),
        Ok(metadata) => {
            ensure!(metadata.is_file(), "missing-result manifest is not a regular file")
        }
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(&path, 16 * 1024 * 1024)
        .with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).map(Some).context("parse missing-result stage manifest")
}

pub(crate) fn publish(
    paths: &MissingResultPaths,
    report: &AllDomainMissingResultTestReport,
) -> Result<()> {
    admit_entry(&paths.repository, &paths.report, false)?;
    bijux_dna_infra::atomic_write_json(&paths.report, report)
        .context("publish missing-result audit")
}
