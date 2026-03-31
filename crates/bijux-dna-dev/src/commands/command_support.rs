use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::model::check::{CheckDefinition, CheckOutcome, CheckStatus};
use crate::runtime::process::ProcessRunner;
use crate::runtime::workspace::Workspace;

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn pass(check: &CheckDefinition, detail: impl Into<String>) -> Result<CheckOutcome> {
    Ok(CheckOutcome::leaf(
        check.id,
        CheckStatus::Passed,
        detail.into(),
    ))
}

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn fail(check: &CheckDefinition, detail: impl Into<String>) -> Result<CheckOutcome> {
    Ok(CheckOutcome::leaf(
        check.id,
        CheckStatus::Failed,
        detail.into(),
    ))
}

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn make_files(workspace: &Workspace) -> Result<Vec<PathBuf>> {
    let mut files = vec![workspace.path("Makefile")];
    for entry in WalkDir::new(workspace.path("makes"))
        .into_iter()
        .filter_map(std::result::Result::ok)
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
    let command_line = std::iter::once(program)
        .chain(args.iter().copied())
        .collect::<Vec<_>>();
    runner.run(&command_line)
}

pub(crate) fn read(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}
