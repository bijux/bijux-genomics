use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use walkdir::WalkDir;

use crate::infrastructure::process::ProcessRunner;
use crate::infrastructure::workspace::Workspace;
use crate::model::check::{CheckDefinition, CheckOutcome, CheckStatus};

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SupportedScriptCatalog {
    #[serde(rename = "script")]
    pub entries: Vec<SupportedScript>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SupportedScript {
    #[serde(rename = "id")]
    pub _id: String,
    pub path: String,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub ci_allowed: bool,
}

pub(crate) fn pass(check: &CheckDefinition, detail: impl Into<String>) -> Result<CheckOutcome> {
    Ok(CheckOutcome::leaf(
        check.id,
        CheckStatus::Passed,
        detail.into(),
    ))
}

pub(crate) fn fail(check: &CheckDefinition, detail: impl Into<String>) -> Result<CheckOutcome> {
    Ok(CheckOutcome::leaf(
        check.id,
        CheckStatus::Failed,
        detail.into(),
    ))
}

pub(crate) fn load_supported_scripts(workspace: &Workspace) -> Result<Vec<SupportedScript>> {
    let spec_path = workspace.path("scripts/SUPPORTED.toml");
    let raw = std::fs::read_to_string(&spec_path)
        .with_context(|| format!("read {}", spec_path.display()))?;
    let parsed: SupportedScriptCatalog =
        toml::from_str(&raw).with_context(|| format!("parse {}", spec_path.display()))?;
    Ok(parsed.entries)
}

pub(crate) fn shell_script_paths(workspace: &Workspace) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(workspace.path("scripts"))
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if !matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("sh") | Some("py")
        ) {
            continue;
        }
        files.push(path.to_path_buf());
    }
    files.sort();
    Ok(files)
}

pub(crate) fn runnable_script_paths(workspace: &Workspace) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for path in shell_script_paths(workspace)? {
        let rel = workspace.rel(&path).to_string_lossy();
        if rel.starts_with("scripts/_lib/")
            || rel.starts_with("scripts/experimental/")
            || rel.starts_with("scripts/tooling/python/")
        {
            continue;
        }
        let meta = std::fs::metadata(&path)
            .with_context(|| format!("read metadata {}", path.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if meta.permissions().mode() & 0o100 == 0 {
                continue;
            }
        }
        files.push(path);
    }
    files.sort();
    Ok(files)
}

pub(crate) fn make_files(workspace: &Workspace) -> Result<Vec<PathBuf>> {
    let mut files = vec![workspace.path("Makefile")];
    for entry in WalkDir::new(workspace.path("makes"))
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("mk") {
            continue;
        }
        files.push(entry.path().to_path_buf());
    }
    files.sort();
    Ok(files)
}

pub(crate) fn run_command(
    workspace: &Workspace,
    program: &str,
    args: &[&str],
) -> Result<std::process::Output> {
    let runner = ProcessRunner::new(workspace);
    let argv = std::iter::once(program)
        .chain(args.iter().copied())
        .collect::<Vec<_>>();
    runner.run(&argv)
}

pub(crate) fn read(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}
