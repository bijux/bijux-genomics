//! Owner: bounded completion-fixture observations, exact mutations and report publication.
use super::{admit_destination, paths::admit_entry, result_root};
use anyhow::{ensure, Context, Result};
use bijux_dna_bench_model::{
    all_domain_simulation_output_ids, all_domain_simulation_output_path,
    completion::{
        AllDomainCompletionCheckReport, AllDomainCompletionRequiredFile,
        DEFAULT_ALL_DOMAIN_COMPLETION_CHECK_FIXTURE_ROOT,
    },
    validate_benchmark_stage_result_manifest, AllDomainSimulationInput,
    BenchmarkStageResultManifest, BenchmarkStageResultStatus,
};
use std::{
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

pub(crate) struct CompletionPaths {
    pub(crate) repository: PathBuf,
    pub(crate) root: PathBuf,
    pub(crate) report: PathBuf,
}

impl CompletionPaths {
    pub(crate) fn display(&self, path: &Path) -> Result<String> {
        path.strip_prefix(&self.repository)
            .unwrap_or(path)
            .to_str()
            .map(str::to_owned)
            .context("completion evidence path must be UTF-8")
    }

    fn fixture_entry(&self, requested: &str, directory: bool) -> Result<PathBuf> {
        let path = admit_destination(&self.repository, Path::new(requested))?;
        ensure!(
            path.starts_with(&self.root) && path != self.root,
            "completion observation escapes its fixture"
        );
        admit_entry(&self.repository, &path, directory)?;
        Ok(path)
    }
}

pub(crate) fn prepare(
    repo: &Path,
    output: &Path,
    inputs: &[AllDomainSimulationInput<'_>],
) -> Result<CompletionPaths> {
    let repository = repo.canonicalize().context("resolve completion repository")?;
    let root = admit_destination(
        &repository,
        Path::new(DEFAULT_ALL_DOMAIN_COMPLETION_CHECK_FIXTURE_ROOT),
    )?;
    let report = if output.is_absolute() { output.to_path_buf() } else { repository.join(output) };
    let paths = CompletionPaths { repository, root, report };
    admit_report(&paths)?;
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

fn admit_report(paths: &CompletionPaths) -> Result<()> {
    ensure!(
        !paths.report.starts_with(&paths.root),
        "completion report must be outside its fixture"
    );
    ensure!(
        !paths.report.components().any(|p| matches!(p, Component::ParentDir)),
        "completion report cannot traverse parents"
    );
    // Absolute report destinations remain supported; inspect their actual ancestors as well.
    let anchor = paths.report.ancestors().last().context("completion report has no root")?;
    admit_entry(anchor, &paths.report, false)
}

pub(crate) fn required_file(
    paths: &CompletionPaths,
    id: &str,
    requested: &str,
) -> Result<AllDomainCompletionRequiredFile> {
    let path = paths.fixture_entry(requested, false)?;
    let observation = inspect_content(&path)?;
    Ok(AllDomainCompletionRequiredFile {
        file_id: id.to_owned(),
        path: paths.display(&path)?,
        exists: observation.is_some(),
        non_empty: observation.unwrap_or(false),
    })
}

pub(crate) fn output_content(
    paths: &CompletionPaths,
    requested: &str,
    id: &str,
) -> Result<Option<bool>> {
    let directory = id.ends_with("_bundle") || id.ends_with("_dir");
    let path = paths.fixture_entry(requested, directory)?;
    inspect_content(&path)
}

fn inspect_content(path: &Path) -> Result<Option<bool>> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("inspect {}", path.display())),
        Ok(metadata) if metadata.is_dir() => {
            Ok(Some(fs::read_dir(path)?.next().transpose()?.is_some()))
        }
        Ok(metadata) => {
            ensure!(metadata.is_file(), "completion evidence is not a regular file");
            Ok(Some(metadata.len() > 0))
        }
    }
}

pub(crate) enum ManifestObservation {
    Missing,
    Invalid,
    Present(Box<BenchmarkStageResultManifest>),
}

pub(crate) fn read_manifest(
    paths: &CompletionPaths,
    requested: &str,
) -> Result<ManifestObservation> {
    let path = paths.fixture_entry(requested, false)?;
    if inspect_content(&path)?.is_none() {
        return Ok(ManifestObservation::Missing);
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(&path, 16 * 1024 * 1024)
        .with_context(|| format!("read {}", path.display()))?;
    let manifest: BenchmarkStageResultManifest = match serde_json::from_slice(&bytes) {
        Ok(manifest) => manifest,
        Err(_) => return Ok(ManifestObservation::Invalid),
    };
    if validate_benchmark_stage_result_manifest(&manifest).is_err() {
        return Ok(ManifestObservation::Invalid);
    }
    Ok(ManifestObservation::Present(Box::new(manifest)))
}

pub(crate) fn remove_file(paths: &CompletionPaths, requested: &str) -> Result<()> {
    let path = paths.fixture_entry(requested, false)?;
    fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))
}

pub(crate) fn empty_file(paths: &CompletionPaths, requested: &str) -> Result<()> {
    let path = paths.fixture_entry(requested, false)?;
    fs::write(&path, b"").with_context(|| format!("empty {}", path.display()))
}

pub(crate) fn fail_execution(paths: &CompletionPaths, requested: &str) -> Result<()> {
    let ManifestObservation::Present(mut manifest) = read_manifest(paths, requested)? else {
        anyhow::bail!("completion failure mutation requires a valid manifest");
    };
    manifest.runtime.exit_code = 23;
    manifest.runtime.status = BenchmarkStageResultStatus::Failed;
    let path = paths.fixture_entry(requested, false)?;
    bijux_dna_infra::atomic_write_json(&path, &manifest)
        .context("publish completion failure mutation")
}

pub(crate) fn publish(
    paths: &CompletionPaths,
    report: &AllDomainCompletionCheckReport,
) -> Result<()> {
    admit_report(paths)?;
    bijux_dna_infra::atomic_write_json(&paths.report, report).context("publish completion report")
}
