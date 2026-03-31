use anyhow::{anyhow, Result};

use crate::catalog::checks::check_registry;
use crate::commands::run_native_check;
use crate::model::check::{
    CheckDefinition, CheckOutcome, CheckSelection, CheckStatus, CommandSpec, ExecutionMode,
};
use crate::runtime::process::ProcessRunner;
use crate::runtime::workspace::Workspace;

#[derive(Debug)]
pub struct CheckApplication {
    workspace: Workspace,
}

impl CheckApplication {
    /// # Errors
    /// Returns an error if the current workspace cannot be resolved.
    pub fn new() -> Result<Self> {
        Ok(Self {
            workspace: Workspace::resolve()?,
        })
    }

    #[must_use]
    pub fn registry() -> Vec<CheckDefinition> {
        check_registry()
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
            CommandSpec::CargoTest {
                package,
                test_bin,
                filter,
            } => self.run_cargo_test(check, package, test_bin, filter),
            CommandSpec::Process { program, args } => self.run_process(check, program, args),
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
                let status = if children
                    .iter()
                    .all(|child| child.status == CheckStatus::Passed)
                {
                    CheckStatus::Passed
                } else {
                    CheckStatus::Failed
                };
                Ok(CheckOutcome::composite(check.id, status, children))
            }
        };
        match outcome {
            Ok(result) => Ok(result),
            Err(error) => Ok(CheckOutcome::leaf(
                check.id,
                CheckStatus::Failed,
                error.to_string(),
            )),
        }
    }

    fn run_cargo_test(
        &self,
        check: &CheckDefinition,
        package: &str,
        test_bin: &str,
        filter: &str,
    ) -> Result<CheckOutcome> {
        let runner = ProcessRunner::new(&self.workspace);
        let output = runner.run(&[
            "cargo", "test", "-p", package, "--test", test_bin, filter, "--quiet",
        ])?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}{stderr}");
        if combined.contains("running 0 tests") {
            return Err(anyhow!(
                "check `{}` matched no tests for filter `{filter}`",
                check.id
            ));
        }
        let status = if output.status.success() {
            CheckStatus::Passed
        } else {
            CheckStatus::Failed
        };
        Ok(CheckOutcome::leaf(check.id, status, combined))
    }

    fn run_process(
        &self,
        check: &CheckDefinition,
        program: &str,
        args: &[&str],
    ) -> Result<CheckOutcome> {
        let runner = ProcessRunner::new(&self.workspace);
        let output = runner.run_owned(
            program,
            &args
                .iter()
                .map(|arg| (*arg).to_string())
                .collect::<Vec<_>>(),
        )?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}{stderr}");
        let status = if output.status.success() {
            CheckStatus::Passed
        } else {
            CheckStatus::Failed
        };
        Ok(CheckOutcome::leaf(check.id, status, combined))
    }
}
