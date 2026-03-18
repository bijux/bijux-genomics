use anyhow::{anyhow, Result};

use crate::infrastructure::workspace::Workspace;
use crate::model::domain::{
    DomainCommandDefinition, DomainCommandOutcome, DomainCommandSpec,
};
use crate::native::run_native_domain_command;
use crate::registry::domain::domain_registry;

#[derive(Debug)]
pub struct DomainApplication {
    workspace: Workspace,
}

impl DomainApplication {
    /// # Errors
    /// Returns an error if the current workspace cannot be resolved.
    pub fn new() -> Result<Self> {
        Ok(Self {
            workspace: Workspace::resolve()?,
        })
    }

    #[must_use]
    pub fn registry(&self) -> Vec<DomainCommandDefinition> {
        domain_registry()
    }

    /// # Errors
    /// Returns an error if the command cannot be resolved or executed.
    pub fn run(&self, id: &str, args: &[String]) -> Result<DomainCommandOutcome> {
        let registry = self.registry();
        let command = registry
            .iter()
            .find(|candidate| candidate.id == id)
            .ok_or_else(|| anyhow!("unknown domain command `{id}`"))?;
        match &command.command {
            DomainCommandSpec::Native { key } => run_native_domain_command(key, &self.workspace, args),
        }
    }
}
