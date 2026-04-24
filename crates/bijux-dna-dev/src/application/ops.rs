use anyhow::{anyhow, Result};

use crate::commands::run_native_ops_command;
use crate::model::ops::{OpsCommandDefinition, OpsCommandOutcome, OpsCommandSpec};
use crate::runtime::workspace::Workspace;

#[derive(Debug)]
pub struct OpsApplication {
    workspace: Workspace,
    registry: fn() -> Vec<OpsCommandDefinition>,
}

impl OpsApplication {
    /// # Errors
    /// Returns an error if the current workspace cannot be resolved.
    pub fn new(registry: fn() -> Vec<OpsCommandDefinition>) -> Result<Self> {
        Ok(Self { workspace: Workspace::resolve()?, registry })
    }

    #[must_use]
    pub fn registry(&self) -> Vec<OpsCommandDefinition> {
        (self.registry)()
    }

    /// # Errors
    /// Returns an error if the command cannot be resolved or executed.
    pub fn run(&self, id: &str, args: &[String]) -> Result<OpsCommandOutcome> {
        let registry = self.registry();
        let command = registry
            .iter()
            .find(|candidate| candidate.id == id)
            .ok_or_else(|| anyhow!("unknown command `{id}`"))?;
        match &command.command {
            OpsCommandSpec::Native { key } => run_native_ops_command(*key, &self.workspace, args),
        }
    }
}
