//! Owner: command artifact destination admission and per-file atomic publication.
use anyhow::{anyhow, ensure, Context, Result};
use std::path::{Component, Path, PathBuf};

pub(crate) struct CommandArtifactPaths {
    pub(crate) script: PathBuf,
    pub(crate) argv: PathBuf,
    pub(crate) script_display: String,
    pub(crate) argv_display: String,
    pub(crate) repo_root_ascent: String,
}

pub(crate) fn prepare(repo_root: &Path, requested: &Path) -> Result<CommandArtifactPaths> {
    ensure!(repo_root.is_dir(), "command artifact repository root must be a directory");
    let script = repo_relative_path(repo_root, requested);
    let repo_root_ascent = repo_root_ascent_from_output(repo_root, &script)?;
    let file_name = script
        .file_name()
        .and_then(|name| name.to_str())
        .context("command artifact output must have a UTF-8 filename")?;
    ensure!(!file_name.is_empty(), "command artifact output filename is empty");
    let stem = file_name.strip_suffix(".sh").unwrap_or(file_name);
    let argv = script.with_file_name(format!("{stem}.argv.jsonl"));
    admit_existing_path(repo_root, &script)?;
    admit_existing_path(repo_root, &argv)?;
    Ok(CommandArtifactPaths {
        script_display: path_relative_to_repo(repo_root, &script),
        argv_display: path_relative_to_repo(repo_root, &argv),
        script,
        argv,
        repo_root_ascent,
    })
}

fn admit_existing_path(root: &Path, path: &Path) -> Result<()> {
    let relative = path.strip_prefix(root).context("command artifact path escapes repository")?;
    let mut current = root.to_path_buf();
    let mut components = relative.components().peekable();
    while let Some(component) = components.next() {
        ensure!(
            matches!(component, Component::Normal(_)),
            "command artifact output must be a normalized repository path"
        );
        current.push(component);
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) => {
                ensure!(
                    !metadata.file_type().is_symlink(),
                    "command artifact path must not traverse a symlink: {}",
                    current.display()
                );
                if components.peek().is_some() {
                    ensure!(
                        metadata.is_dir(),
                        "command artifact ancestor is not a directory: {}",
                        current.display()
                    );
                } else {
                    ensure!(
                        metadata.is_file(),
                        "command artifact destination is not a regular file: {}",
                        current.display()
                    );
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("inspect {}", current.display()))
            }
        }
    }
    Ok(())
}

pub(crate) fn publish(paths: &CommandArtifactPaths, script: &str, argv: &str) -> Result<()> {
    bijux_dna_infra::atomic_write_bytes(&paths.script, script.as_bytes())?;
    bijux_dna_infra::atomic_write_bytes(&paths.argv, argv.as_bytes())?;
    Ok(())
}

fn repo_root_ascent_from_output(repo_root: &Path, output_path: &Path) -> Result<String> {
    let output_parent = output_path.parent().ok_or_else(|| {
        anyhow!("all-domain rendered command output must have a parent directory")
    })?;
    let relative_parent = output_parent.strip_prefix(repo_root).with_context(|| {
        format!(
            "all-domain rendered command output `{}` must stay within repository root `{}`",
            output_path.display(),
            repo_root.display()
        )
    })?;
    if relative_parent
        .components()
        .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(anyhow!(
            "all-domain rendered command output parent must be a normalized repository path"
        ));
    }
    let depth = relative_parent.components().count();
    Ok(if depth == 0 {
        ".".to_string()
    } else {
        (0..depth).map(|_| "..").collect::<Vec<_>>().join("/")
    })
}
fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}
fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

pub(crate) mod slurm_scripts;

pub(crate) mod slurm_dependencies;
