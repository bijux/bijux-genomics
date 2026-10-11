//! Owner: benchmark script inventory observation and report publication.
use anyhow::{ensure, Context, Result};
use bijux_dna_bench_model::BenchmarkLocalSlurmScriptBodyReport;
use std::path::{Component, Path, PathBuf};

pub(crate) struct InspectionPaths {
    pub(crate) repository: PathBuf,
    pub(crate) root: PathBuf,
    pub(crate) report: PathBuf,
}
impl InspectionPaths {
    pub(crate) fn display(&self, path: &Path) -> String {
        path.strip_prefix(&self.repository).unwrap_or(path).to_string_lossy().replace('\\', "/")
    }
}

pub(crate) fn prepare(
    repo_root: &Path,
    requested_root: &Path,
    requested_report: &Path,
) -> Result<InspectionPaths> {
    let repository = repo_root.canonicalize().context("resolve script inspection repository")?;
    let absolute =
        |path: &Path| if path.is_absolute() { path.to_path_buf() } else { repository.join(path) };
    let root = absolute(requested_root);
    let report = absolute(requested_report);
    let allowed = repository.join("artifacts/runs/bench");
    for (path, kind) in
        [(&root, "slurm dry-run root"), (&report, "slurm script body report output")]
    {
        ensure!(path.starts_with(&allowed), "{kind} must resolve inside `artifacts/runs/bench`");
        let relative = path.strip_prefix(&repository)?;
        ensure!(
            relative.components().all(|part| matches!(part, Component::Normal(_))),
            "script inspection requires normalized repository paths"
        );
    }
    super::admit_existing_path(&repository, &report)?;
    Ok(InspectionPaths { repository, root, report })
}

pub(crate) fn discover_scripts(paths: &InspectionPaths) -> Result<Vec<PathBuf>> {
    match std::fs::symlink_metadata(&paths.root) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error).context("inspect script inventory root"),
        Ok(metadata) => ensure!(
            metadata.is_dir() && !metadata.file_type().is_symlink(),
            "script inventory root must be a regular directory"
        ),
    }
    ensure!(
        paths.root.canonicalize()? == paths.root,
        "script inventory must not traverse symlink ancestors"
    );
    let mut directories = vec![paths.root.clone()];
    let mut scripts = Vec::new();
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(&directory)
            .with_context(|| format!("read {}", directory.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let kind = entry.file_type()?;
            ensure!(
                !kind.is_symlink(),
                "script inventory must not contain symlinks: {}",
                path.display()
            );
            if kind.is_dir() {
                directories.push(path);
            } else if path.extension().is_some_and(|extension| extension == "sbatch") {
                ensure!(kind.is_file(), "script inventory entry is not a regular file");
                scripts.push(path);
            }
        }
    }
    scripts.sort();
    Ok(scripts)
}

pub(crate) fn read_body(path: &Path) -> Result<String> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, 16 * 1024 * 1024)
        .with_context(|| format!("read {}", path.display()))?;
    String::from_utf8(bytes).context("script body is not UTF-8")
}

pub(crate) fn publish(
    paths: &InspectionPaths,
    report: &BenchmarkLocalSlurmScriptBodyReport,
) -> Result<()> {
    super::admit_existing_path(&paths.repository, &paths.report)?;
    bijux_dna_infra::atomic_write_json(&paths.report, report)
        .context("publish Slurm script inspection report")
}
