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
        Ok(Self { workspace: Workspace::resolve()? })
    }

    #[must_use]
    pub fn registry() -> Vec<DomainCommandDefinition> {
        normalize_registry(domain_registry())
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

fn normalize_registry(mut registry: Vec<DomainCommandDefinition>) -> Vec<DomainCommandDefinition> {
    registry.sort_by(|left, right| left.id.cmp(&right.id));
    for pair in registry.windows(2) {
        assert_ne!(pair[0].id, pair[1].id, "duplicate domain command id `{}`", pair[0].id);
    }
    registry
}

#[cfg(test)]
mod tests {
    use crate::model::domain::{
        DomainCommandDefinition, DomainCommandSpec, NativeDomainCommandKey,
    };

    use super::normalize_registry;

    #[test]
    fn normalizes_domain_registry_order() {
        let registry = normalize_registry(vec![command("zeta"), command("alpha")]);

        let ids = registry.into_iter().map(|command| command.id).collect::<Vec<_>>();
        assert_eq!(ids, ["alpha", "zeta"]);
    }

    #[test]
    #[should_panic(expected = "duplicate domain command id `alpha`")]
    fn rejects_duplicate_domain_command_ids() {
        let _registry = normalize_registry(vec![command("alpha"), command("alpha")]);
    }

    fn command(id: &str) -> DomainCommandDefinition {
        DomainCommandDefinition {
            id: id.to_string(),
            summary: String::new(),
            command: DomainCommandSpec::Native { key: NativeDomainCommandKey::Validate },
        }
    }
}
