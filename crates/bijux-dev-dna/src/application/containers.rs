use anyhow::{anyhow, Result};

use crate::infrastructure::workspace::Workspace;
use crate::model::container::{ContainerCommandDefinition, ContainerCommandOutcome, ContainerCommandSpec};
use crate::native::run_native_container_command;
use crate::registry::containers::container_registry;

#[derive(Debug)]
pub struct ContainerApplication {
    workspace: Workspace,
}

impl ContainerApplication {
    /// # Errors
    /// Returns an error if the current workspace cannot be resolved.
    pub fn new() -> Result<Self> {
        Ok(Self {
            workspace: Workspace::resolve()?,
        })
    }

    /// # Errors
    /// Returns an error if the container command registry cannot be resolved.
    pub fn registry(&self) -> Result<Vec<ContainerCommandDefinition>> {
        container_registry(&self.workspace)
    }

    /// # Errors
    /// Returns an error if the command cannot be resolved or executed.
    pub fn run(&self, id: &str, args: &[String]) -> Result<ContainerCommandOutcome> {
        let registry = self.registry()?;
        let command = registry
            .iter()
            .find(|candidate| candidate.id == id)
            .ok_or_else(|| anyhow!("unknown container command `{id}`"))?;
        match &command.command {
            ContainerCommandSpec::Native { key } => run_native_container_command(key, &self.workspace, args),
        }
    }
}
