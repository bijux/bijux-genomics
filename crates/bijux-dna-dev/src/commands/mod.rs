mod automation_boundary;
mod checks;
mod command_support;
mod containers;
mod domain;
mod ops;
mod repo_checks;

use anyhow::Result;

use crate::model::check::{CheckDefinition, CheckOutcome, NativeCheckKey};
use crate::model::container::{ContainerCommandOutcome, NativeContainerCommandKey};
use crate::model::domain::{DomainCommandOutcome, NativeDomainCommandKey};
use crate::model::ops::{NativeOpsCommandKey, OpsCommandOutcome};
use crate::runtime::workspace::Workspace;

/// # Errors
/// Returns an error if the native check cannot run.
pub fn run_native_check(
    key: &NativeCheckKey,
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    checks::run_native_check(key, workspace, check)
}

/// # Errors
/// Returns an error if the native container command cannot run.
pub fn run_native_container_command(
    key: &NativeContainerCommandKey,
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    containers::run_native_container_command(key, workspace, args)
}

/// # Errors
/// Returns an error if the native domain command cannot run.
pub fn run_native_domain_command(
    key: &NativeDomainCommandKey,
    workspace: &Workspace,
    args: &[String],
) -> Result<DomainCommandOutcome> {
    domain::run_native_domain_command(key, workspace, args)
}

/// # Errors
/// Returns an error if the native operational command cannot run.
pub fn run_native_ops_command(
    key: &NativeOpsCommandKey,
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ops::run_native_ops_command(key, workspace, args)
}
