use anyhow::{anyhow, Result};

use crate::catalog::checks::check_registry;
use crate::commands::run_native_check;
use crate::model::check::{
    CheckDefinition, CheckOutcome, CheckSelection, CheckStatus, CommandSpec, ExecutionMode,
};
use crate::runtime::workspace::Workspace;

mod execution_adapters;
mod policy_observations;

use policy_observations::{CargoTestKey, CargoTestObservation, RequestPolicyObservations};

trait CheckExecutor {
    fn observe_cargo(
        &mut self,
        workspace: &Workspace,
        key: CargoTestKey,
    ) -> Result<CargoTestObservation>;
    fn execute_effect(
        &mut self,
        workspace: &Workspace,
        check: &CheckDefinition,
    ) -> Result<CheckOutcome>;
}

struct ProductionExecution;

impl CheckExecutor for ProductionExecution {
    fn observe_cargo(
        &mut self,
        workspace: &Workspace,
        key: CargoTestKey,
    ) -> Result<CargoTestObservation> {
        execution_adapters::observe_cargo_test(workspace, key.package, key.test_bin, key.filter)
    }

    fn execute_effect(
        &mut self,
        workspace: &Workspace,
        check: &CheckDefinition,
    ) -> Result<CheckOutcome> {
        match &check.command {
            CommandSpec::Process { program, args } => {
                execution_adapters::run_process(workspace, check, program, args)
            }
            CommandSpec::Native { key } => run_native_check(*key, workspace, check),
            _ => Err(anyhow!("expected an effectful check `{}`", check.id)),
        }
    }
}

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
        self.run_selection_with(&Self::registry(), selection, &mut ProductionExecution)
    }

    fn run_selection_with<E: CheckExecutor>(
        &self,
        registry: &[CheckDefinition],
        selection: CheckSelection,
        executor: &mut E,
    ) -> Result<Vec<CheckOutcome>> {
        let mut observations = RequestPolicyObservations::default();
        match selection {
            CheckSelection::All => registry
                .iter()
                .filter(|check| check.execution_mode == ExecutionMode::Primary)
                .map(|check| self.run_check(registry, check, executor, &mut observations))
                .collect(),
            CheckSelection::Single(id) => {
                let check = registry
                    .iter()
                    .find(|candidate| {
                        candidate.id == id || candidate.aliases.contains(&id.as_str())
                    })
                    .ok_or_else(|| anyhow!("unknown check id `{id}`"))?;
                Ok(vec![self.run_check(registry, check, executor, &mut observations)?])
            }
        }
    }

    fn run_check<E: CheckExecutor>(
        &self,
        registry: &[CheckDefinition],
        check: &CheckDefinition,
        executor: &mut E,
        observations: &mut RequestPolicyObservations,
    ) -> Result<CheckOutcome> {
        let outcome = match &check.command {
            CommandSpec::CargoTest { package, test_bin, filter } => {
                let key = CargoTestKey { package, test_bin, filter };
                Ok(observations
                    .observe(key, || executor.observe_cargo(&self.workspace, key))
                    .for_check(check.id))
            }
            CommandSpec::Process { .. } | CommandSpec::Native { .. } => {
                observations.clear();
                let result = executor.execute_effect(&self.workspace, check);
                observations.clear();
                result
            }
            CommandSpec::Composite { members } => {
                observations.clear();
                let mut children = Vec::new();
                for member in *members {
                    let nested = registry
                        .iter()
                        .find(|candidate| candidate.id == *member)
                        .ok_or_else(|| anyhow!("missing composite member `{member}`"))?;
                    children.push(self.run_check(registry, nested, executor, observations)?);
                }
                observations.clear();
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

#[cfg(test)]
mod request_observation_contracts;
