use anyhow::{anyhow, Result};

use crate::catalog::domain::domain_registry;
use crate::commands::run_native_domain_command;
use crate::model::domain::{DomainCommandDefinition, DomainCommandOutcome, DomainCommandSpec};
use crate::runtime::workspace::Workspace;

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
    pub fn registry() -> Vec<DomainCommandDefinition> {
        domain_registry()
    }

    /// # Errors
    /// Returns an error if the command cannot be resolved or executed.
    pub fn run(&self, id: &str, args: &[String]) -> Result<DomainCommandOutcome> {
        let registry = Self::registry();
        let command = registry
            .iter()
            .find(|candidate| candidate.id == id)
            .ok_or_else(|| anyhow!("unknown domain command `{id}`"))?;
        match &command.command {
            DomainCommandSpec::Native { key } => {
                run_native_domain_command(*key, &self.workspace, args)
            }
        }
    }
}
