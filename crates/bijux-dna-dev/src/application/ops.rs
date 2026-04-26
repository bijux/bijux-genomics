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
        normalize_registry((self.registry)())
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

fn normalize_registry(mut registry: Vec<OpsCommandDefinition>) -> Vec<OpsCommandDefinition> {
    registry.sort_by(|left, right| left.id.cmp(&right.id));
    for pair in registry.windows(2) {
        assert_ne!(pair[0].id, pair[1].id, "duplicate ops command id `{}`", pair[0].id);
    }
    registry
}

#[cfg(test)]
mod tests {
    use crate::model::ops::{NativeOpsCommandKey, OpsCommandDefinition, OpsCommandSpec};

    use super::normalize_registry;

    #[test]
    fn normalizes_ops_registry_order() {
        let registry = normalize_registry(vec![command("zeta"), command("alpha")]);

        let ids = registry.into_iter().map(|command| command.id).collect::<Vec<_>>();
        assert_eq!(ids, ["alpha", "zeta"]);
    }

    #[test]
    #[should_panic(expected = "duplicate ops command id `alpha`")]
    fn rejects_duplicate_ops_command_ids() {
        let _registry = normalize_registry(vec![command("alpha"), command("alpha")]);
    }

    fn command(id: &str) -> OpsCommandDefinition {
        OpsCommandDefinition {
            id: id.to_string(),
            summary: String::new(),
            command: OpsCommandSpec::Native { key: NativeOpsCommandKey::SmokeRun },
        }
    }
}
