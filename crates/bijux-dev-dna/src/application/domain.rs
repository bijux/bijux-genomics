use anyhow::{anyhow, Result};

use crate::infrastructure::process::ProcessRunner;
use crate::infrastructure::workspace::Workspace;
use crate::model::domain::{DomainCommandDefinition, DomainCommandOutcome};
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

    /// # Errors
    /// Returns an error if the domain command registry cannot be resolved.
    pub fn registry(&self) -> Result<Vec<DomainCommandDefinition>> {
        domain_registry(&self.workspace)
    }

    /// # Errors
    /// Returns an error if the command cannot be resolved or executed.
    pub fn run(&self, id: &str, args: &[String]) -> Result<DomainCommandOutcome> {
        let registry = self.registry()?;
        let command = registry
            .iter()
            .find(|candidate| candidate.id == id)
            .ok_or_else(|| anyhow!("unknown domain command `{id}`"))?;
        let runner = ProcessRunner::new(&self.workspace);
        let program = format!("./{}", command.rel_path);
        let output = runner.run_owned(&program, args)?;
        Ok(DomainCommandOutcome::from_output(output))
    }
}
