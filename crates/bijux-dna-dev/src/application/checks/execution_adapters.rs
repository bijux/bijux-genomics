use anyhow::{anyhow, Result};

use crate::model::check::{CheckDefinition, CheckOutcome, CheckStatus};
use crate::runtime::process::ProcessRunner;
use crate::runtime::workspace::Workspace;

pub(super) fn run_cargo_test(
    workspace: &Workspace,
    check: &CheckDefinition,
    package: &str,
    test_bin: &str,
    filter: &str,
) -> Result<CheckOutcome> {
    let runner = ProcessRunner::new(workspace);
    let output = runner.run(&[
        "cargo", "test", "-p", package, "--test", test_bin, filter, "--quiet",
    ])?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    if combined.contains("running 0 tests") {
        return Err(anyhow!(
            "check `{}` matched no tests for filter `{filter}`",
            check.id
        ));
    }
    let status = if output.status.success() {
        CheckStatus::Passed
    } else {
        CheckStatus::Failed
    };
    Ok(CheckOutcome::leaf(check.id, status, combined))
}

pub(super) fn run_process(
    workspace: &Workspace,
    check: &CheckDefinition,
    program: &str,
    args: &[&str],
) -> Result<CheckOutcome> {
    let runner = ProcessRunner::new(workspace);
    let output = runner.run_owned(
        program,
        &args
            .iter()
            .map(|arg| (*arg).to_string())
            .collect::<Vec<_>>(),
    )?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");
    let status = if output.status.success() {
        CheckStatus::Passed
    } else {
        CheckStatus::Failed
    };
    Ok(CheckOutcome::leaf(check.id, status, combined))
}
