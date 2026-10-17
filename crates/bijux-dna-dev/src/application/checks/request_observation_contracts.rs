#![cfg(test)]

use super::*;
use std::collections::BTreeMap;

struct ObservedExecution {
    calls: Vec<CargoTestKey>,
    effects: Vec<String>,
    result: CargoTestObservation,
    input: Option<std::path::PathBuf>,
}

impl ObservedExecution {
    fn passing() -> Self {
        Self {
            calls: Vec::new(),
            effects: Vec::new(),
            result: CargoTestObservation::Completed {
                status: CheckStatus::Passed,
                detail: "qualified observation".to_string(),
            },
            input: None,
        }
    }
}

impl CheckExecutor for ObservedExecution {
    fn observe_cargo(
        &mut self,
        _workspace: &Workspace,
        key: CargoTestKey,
    ) -> Result<CargoTestObservation> {
        self.calls.push(key);
        if key.filter == "unknown_policy" {
            if let Some(input) = &self.input {
                std::fs::write(input, "changed")?;
            }
        }
        if let Some(input) = &self.input {
            return Ok(CargoTestObservation::Completed {
                status: CheckStatus::Passed,
                detail: std::fs::read_to_string(input)?,
            });
        }
        Ok(self.result.clone())
    }
    fn execute_effect(
        &mut self,
        _workspace: &Workspace,
        check: &CheckDefinition,
    ) -> Result<CheckOutcome> {
        self.effects.push(check.id.to_string());
        if let Some(input) = &self.input {
            std::fs::write(input, "changed")?;
        }
        Ok(CheckOutcome::leaf(check.id, CheckStatus::Passed, "effect observed".to_string()))
    }
}

fn app(root: &std::path::Path) -> CheckApplication {
    CheckApplication { workspace: Workspace { root: root.to_path_buf() } }
}
fn policy(id: &'static str, filter: &'static str) -> CheckDefinition {
    CheckDefinition {
        id,
        version: 1,
        summary: String::new(),
        aliases: &[],
        execution_mode: ExecutionMode::Primary,
        command: CommandSpec::CargoTest {
            package: "bijux-dna-policies",
            test_bin: "contracts",
            filter,
        },
    }
}
fn effect(id: &'static str) -> CheckDefinition {
    CheckDefinition {
        command: CommandSpec::Native { key: crate::model::check::NativeCheckKey::AutomationIntent },
        ..policy(id, "unused")
    }
}

#[test]
fn primary_policy_groups_observe_once_between_effect_boundaries() -> Result<()> {
    let root = tempfile::tempdir()?;
    let registry = CheckApplication::registry();
    let mut executor = ObservedExecution::passing();
    let outcomes =
        app(root.path()).run_selection_with(&registry, CheckSelection::All, &mut executor)?;
    let expected_ids = registry
        .iter()
        .filter(|check| check.execution_mode == ExecutionMode::Primary)
        .map(|check| check.id)
        .collect::<Vec<_>>();
    assert_eq!(
        outcomes.iter().map(|outcome| outcome.id.as_str()).collect::<Vec<_>>(),
        expected_ids
    );
    for outcome in &outcomes {
        let definition = registry
            .iter()
            .find(|check| check.id == outcome.id)
            .ok_or_else(|| anyhow!("outcome missing registry owner"))?;
        if let CommandSpec::Composite { members } = &definition.command {
            assert_eq!(
                outcome.children.iter().map(|child| child.id.as_str()).collect::<Vec<_>>(),
                *members
            );
        }
    }
    let expected_effects = registry
        .iter()
        .filter(|check| check.execution_mode == ExecutionMode::Primary)
        .filter(|check| {
            matches!(check.command, CommandSpec::Native { .. } | CommandSpec::Process { .. })
        })
        .map(|check| check.id)
        .collect::<Vec<_>>();
    assert_eq!(executor.effects.iter().map(String::as_str).collect::<Vec<_>>(), expected_effects);
    use std::io::Write as _;
    writeln!(
        std::io::stderr().lock(),
        "policy request observation: {} Cargo dispatch requests; trace={:?}",
        executor.calls.len(),
        executor.calls,
    )?;
    let counts = executor.calls.iter().fold(BTreeMap::new(), |mut counts, key| {
        *counts.entry(key.filter).or_insert(0usize) += 1;
        counts
    });
    assert_eq!(
        counts["assets_governance_policy"], 2,
        "separate effect stretches must remain fresh; physical dispatch trace has {} requests: {:?}",
        executor.calls.len(), executor.calls
    );
    assert_eq!(counts["generated_configs_policy"], 2);
    assert_eq!(counts["vcf_support_gate_policy"], 4);
    assert_eq!(counts["examples_cli_command_policy"], 3);
    assert_eq!(
        counts["stage_executor_parity_policy"], 2,
        "explicit composite child remains executable"
    );
    assert!(outcomes.iter().all(|outcome| outcome.status == CheckStatus::Passed));
    Ok(())
}

#[test]
fn policy_observations_are_invalidated_by_intervening_effects() -> Result<()> {
    let root = tempfile::tempdir()?;
    let input = root.path().join("policy-input.txt");
    let boundaries = [
        effect("intervening-native"),
        CheckDefinition {
            command: CommandSpec::Process { program: "production-effect", args: &[] },
            ..effect("intervening-process")
        },
        policy("intervening-unknown", "unknown_policy"),
    ];
    for boundary in boundaries {
        std::fs::write(&input, "original")?;
        let registry = vec![
            policy("first", "assets_governance_policy"),
            boundary,
            policy("later", "assets_governance_policy"),
        ];
        let mut executor = ObservedExecution::passing();
        executor.input = Some(input.clone());
        let outcomes =
            app(root.path()).run_selection_with(&registry, CheckSelection::All, &mut executor)?;
        assert_eq!(outcomes[0].detail, "original");
        assert_eq!(outcomes[2].detail, "changed");
        assert_eq!(
            executor.calls.iter().filter(|key| key.filter == "assets_governance_policy").count(),
            2
        );
    }
    Ok(())
}

#[test]
fn policy_failures_keep_each_primary_and_selected_alias_identity() -> Result<()> {
    let root = tempfile::tempdir()?;
    let registry = vec![
        policy("first", "assets_governance_policy"),
        policy("later", "assets_governance_policy"),
    ];
    for result in [
        CargoTestObservation::Completed {
            status: CheckStatus::Failed,
            detail: "actual policy failure".to_string(),
        },
        CargoTestObservation::NoTests { filter: "assets_governance_policy".to_string() },
        CargoTestObservation::LaunchFailure {
            detail: "run cargo test: refused provider".to_string(),
        },
    ] {
        let mut executor = ObservedExecution::passing();
        executor.result = result;
        let outcomes =
            app(root.path()).run_selection_with(&registry, CheckSelection::All, &mut executor)?;
        assert_eq!(
            outcomes.iter().map(|outcome| outcome.id.as_str()).collect::<Vec<_>>(),
            ["first", "later"]
        );
        assert!(outcomes.iter().all(|outcome| outcome.status == CheckStatus::Failed));
        if matches!(executor.result, CargoTestObservation::NoTests { .. }) {
            for outcome in &outcomes {
                assert_eq!(
                    outcome.detail,
                    format!(
                        "check `{}` matched no tests for filter `assets_governance_policy`",
                        outcome.id
                    )
                );
            }
        } else {
            assert_eq!(outcomes[0].detail, outcomes[1].detail);
        }
        assert_eq!(executor.calls.len(), 1);
    }
    let actual = CheckApplication::registry();
    let alias = actual
        .iter()
        .find(|check| check.id == "check-stage-executor-parity")
        .ok_or_else(|| anyhow!("actual alias absent"))?;
    assert_eq!(alias.execution_mode, ExecutionMode::Alias);
    let mut executor = ObservedExecution::passing();
    executor.result = CargoTestObservation::Completed {
        status: CheckStatus::Failed,
        detail: "actual alias failure".to_string(),
    };
    let outcomes = app(root.path()).run_selection_with(
        &actual,
        CheckSelection::Single(alias.id.to_string()),
        &mut executor,
    )?;
    assert_eq!(outcomes[0].id, alias.id);
    assert_eq!(outcomes[0].status, CheckStatus::Failed);
    assert_eq!(executor.calls.len(), 1);
    Ok(())
}

#[test]
fn second_check_request_reobserves_private_inputs() -> Result<()> {
    let root = tempfile::tempdir()?;
    let input = root.path().join("input.txt");
    let registry =
        [policy("first", "assets_governance_policy"), policy("later", "assets_governance_policy")];
    let application = app(root.path());
    let mut executor = ObservedExecution::passing();
    executor.input = Some(input.clone());
    for value in ["original", "changed"] {
        std::fs::write(&input, value)?;
        let outcomes =
            application.run_selection_with(&registry, CheckSelection::All, &mut executor)?;
        assert!(outcomes.iter().all(|outcome| outcome.detail == value));
    }
    assert_eq!(
        executor.calls.len(),
        2,
        "observations must be request-owned, not application-owned"
    );
    Ok(())
}

#[test]
fn future_and_opposite_policy_commands_are_fresh() -> Result<()> {
    let root = tempfile::tempdir()?;
    let known = policy("known", "assets_governance_policy");
    let mut foreign_package = known.clone();
    foreign_package.id = "foreign-package";
    foreign_package.command = CommandSpec::CargoTest {
        package: "foreign",
        test_bin: "contracts",
        filter: "assets_governance_policy",
    };
    let mut foreign_binary = known.clone();
    foreign_binary.id = "foreign-binary";
    foreign_binary.command = CommandSpec::CargoTest {
        package: "bijux-dna-policies",
        test_bin: "other",
        filter: "assets_governance_policy",
    };
    for unknown in
        [foreign_package, foreign_binary, policy("future", "future_assets_governance_policy")]
    {
        let registry =
            [known.clone(), unknown.clone(), unknown, policy("again", "assets_governance_policy")];
        let mut executor = ObservedExecution::passing();
        app(root.path()).run_selection_with(&registry, CheckSelection::All, &mut executor)?;
        assert_eq!(executor.calls.len(), 4, "unknown tuples invalidate and always execute");
    }
    let second_root = tempfile::tempdir()?;
    let registry = [known];
    let mut executor = ObservedExecution::passing();
    for root_path in [root.path(), second_root.path()] {
        app(root_path).run_selection_with(&registry, CheckSelection::All, &mut executor)?;
    }
    assert_eq!(executor.calls.len(), 2);
    Ok(())
}

#[test]
fn composite_boundaries_keep_children_and_invalidate_both_sides() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut child = policy("child", "assets_governance_policy");
    child.execution_mode = ExecutionMode::Alias;
    let registry = [
        policy("first", "assets_governance_policy"),
        CheckDefinition {
            command: CommandSpec::Composite { members: &["child"] },
            ..effect("composite")
        },
        child,
        policy("later", "assets_governance_policy"),
    ];
    let mut executor = ObservedExecution::passing();
    let outcomes =
        app(root.path()).run_selection_with(&registry, CheckSelection::All, &mut executor)?;
    assert_eq!(outcomes[1].children[0].id, "child");
    assert_eq!(executor.calls.len(), 3);
    let missing = [CheckDefinition {
        command: CommandSpec::Composite { members: &["missing"] },
        ..effect("composite")
    }];
    let Err(error) =
        app(root.path()).run_selection_with(&missing, CheckSelection::All, &mut executor)
    else {
        return Err(anyhow!("missing composite member must refuse"));
    };
    assert_eq!(error.to_string(), "missing composite member `missing`");
    Ok(())
}
