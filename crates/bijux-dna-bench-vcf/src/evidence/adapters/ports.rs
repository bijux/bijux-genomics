use crate::evidence::scope::ActiveBindingProvider;
use anyhow::Result;
use std::path::Path;
/// Observed shell parser exit status and diagnostics; never an execution-success claim.
pub struct ShellSyntaxObservation {
    pub exit_code: i32,
    pub stderr: String,
}
/// Composition supplies the active binding authority and a shell syntax inspector.
/// The inspector receives the repository and published script path; it must inspect syntax only.
#[derive(Clone, Copy)]
pub struct VcfAdapterReadinessPorts {
    pub active_bindings: ActiveBindingProvider,
    pub inspect_shell: fn(&Path, &Path) -> Result<ShellSyntaxObservation>,
}
