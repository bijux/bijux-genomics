use anyhow::{anyhow, Result};

use crate::catalog::checks::check_registry;
use crate::commands::run_native_check;
use crate::model::check::{
    CheckDefinition, CheckOutcome, CheckSelection, CheckStatus, CommandSpec, ExecutionMode,
};
use crate::runtime::workspace::Workspace;

mod execution_adapters;

#[derive(Debug)]
pub struct CheckApplication {
    workspace: Workspace,
}

impl CheckApplication {
    /// # Errors
    /// Returns an error if the current workspace cannot be resolved.
    pub fn new() -> Result<Self> {
        Ok(Self { workspace: Workspace::resolve()? })
    }

    #[must_use]
    pub fn registry() -> Vec<CheckDefinition> {
        normalize_registry(check_registry())
    }

    /// # Errors
    /// Returns an error if registry resolution or check execution fails.
    pub fn run_selection(&self, selection: CheckSelection) -> Result<Vec<CheckOutcome>> {
        let registry = Self::registry();
        match selection {
            CheckSelection::All => registry
                .iter()
                .filter(|check| check.execution_mode == ExecutionMode::Primary)
                .map(|check| self.run_check(&registry, check))
                .collect(),
            CheckSelection::Single(id) => {
                let check = registry
                    .iter()
                    .find(|candidate| {
                        candidate.id == id || candidate.aliases.contains(&id.as_str())
                    })
                    .ok_or_else(|| anyhow!("unknown check id `{id}`"))?;
                Ok(vec![self.run_check(&registry, check)?])
            }
        }
    }

    fn run_check(
        &self,
        registry: &[CheckDefinition],
        check: &CheckDefinition,
    ) -> Result<CheckOutcome> {
        let outcome = match &check.command {
            CommandSpec::CargoTest { package, test_bin, filter } => {
                execution_adapters::run_cargo_test(
                    &self.workspace,
                    check,
                    package,
                    test_bin,
                    filter,
                )
            }
            CommandSpec::Process { program, args } => {
                execution_adapters::run_process(&self.workspace, check, program, args)
            }
            CommandSpec::Native { key } => run_native_check(*key, &self.workspace, check),
            CommandSpec::Composite { members } => {
                let mut children = Vec::new();
                for member in *members {
                    let nested = registry
                        .iter()
                        .find(|candidate| candidate.id == *member)
                        .ok_or_else(|| anyhow!("missing composite member `{member}`"))?;
                    children.push(self.run_check(registry, nested)?);
                }
                let status = if children.iter().all(|child| child.status == CheckStatus::Passed) {
                    CheckStatus::Passed
                } else {
                    CheckStatus::Failed
                };
                Ok(CheckOutcome::composite(check.id, status, children))
            }
        };
        match outcome {
            Ok(result) => Ok(result),
            Err(error) => Ok(CheckOutcome::leaf(check.id, CheckStatus::Failed, error.to_string())),
        }
    }
}

fn normalize_registry(mut registry: Vec<CheckDefinition>) -> Vec<CheckDefinition> {
    registry.sort_by(|left, right| left.id.cmp(right.id));
    for pair in registry.windows(2) {
        assert_ne!(pair[0].id, pair[1].id, "duplicate check id `{}`", pair[0].id);
    }
    registry
}

#[cfg(test)]
mod tests {
    use crate::model::check::{CheckDefinition, CommandSpec, ExecutionMode, NativeCheckKey};

    use super::normalize_registry;

    #[test]
    fn normalizes_check_registry_order() {
        let registry = normalize_registry(vec![check("check-zeta"), check("check-alpha")]);

        let ids = registry.into_iter().map(|check| check.id).collect::<Vec<_>>();
        assert_eq!(ids, ["check-alpha", "check-zeta"]);
    }

    #[test]
    #[should_panic(expected = "duplicate check id `check-alpha`")]
    fn rejects_duplicate_check_ids() {
        let _registry = normalize_registry(vec![check("check-alpha"), check("check-alpha")]);
    }

    fn check(id: &'static str) -> CheckDefinition {
        CheckDefinition {
            id,
            version: 1,
            summary: String::new(),
            aliases: &[],
            execution_mode: ExecutionMode::Primary,
            command: CommandSpec::Native { key: NativeCheckKey::AutomationIntent },
        }
    }
}
