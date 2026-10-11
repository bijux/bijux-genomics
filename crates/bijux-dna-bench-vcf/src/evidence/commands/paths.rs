use super::DEFAULT_VCF_RENDERED_COMMANDS_PATH;
use std::path::{Path, PathBuf};
pub(super) fn companion_argv_output_path(output_path: &Path) -> PathBuf {
    let file_name = output_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(DEFAULT_VCF_RENDERED_COMMANDS_PATH);
    let argv_name = if let Some(stem) = file_name.strip_suffix(".sh") {
        format!("{stem}.argv.jsonl")
    } else {
        format!("{file_name}.argv.jsonl")
    };
    output_path.with_file_name(argv_name)
}
pub(super) fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}
pub(super) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
pub(super) fn repo_root_relative_to_output(repo_root: &Path, output_path: &Path) -> String {
    let relative_output_path = output_path.strip_prefix(repo_root).unwrap_or(output_path);
    let depth = relative_output_path.parent().map_or(0, |parent| parent.components().count());
    if depth == 0 {
        ".".to_string()
    } else {
        std::iter::repeat_n("..", depth).collect::<Vec<_>>().join("/")
    }
}

pub(super) enum ScriptRepositoryRoot {
    Relative(String),
    Absolute(String),
}
pub(super) fn script_repository_root(
    repo_root: &Path,
    output_path: &Path,
) -> anyhow::Result<ScriptRepositoryRoot> {
    use anyhow::Context;
    let is_owned = output_path.strip_prefix(repo_root).is_ok_and(|path| {
        !path.components().any(|part| matches!(part, std::path::Component::ParentDir))
    });
    if is_owned {
        return Ok(ScriptRepositoryRoot::Relative(repo_root_relative_to_output(
            repo_root,
            output_path,
        )));
    }
    let canonical = repo_root
        .canonicalize()
        .with_context(|| format!("resolve script repository {}", repo_root.display()))?;
    let path = canonical.to_str().context("script repository path must be valid UTF-8")?;
    Ok(ScriptRepositoryRoot::Absolute(path.to_owned()))
}
