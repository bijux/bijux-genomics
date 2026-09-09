use anyhow::{ensure, Context, Result};
use serde_json::Value;

use super::{run_program_with_env, write_json_pretty, OpsCommandOutcome, Workspace};

const GROUPS: &[(&str, &str)] = &[
    ("boundaries", "mod_boundaries_deps_dependency_graph_rs::"),
    ("boundaries", "mod_boundaries_purity_scans_rs::"),
    ("boundaries", "mod_boundaries_deps_core_layering_rs::"),
    ("boundaries", "mod_boundaries_deps_domain_dependency_policy_rs::"),
    ("contracts", "mod_contracts_tooling_ci_tools_policy_rs::"),
    ("boundaries", "mod_boundaries_deps_dev_deps_policy_rs::"),
    ("boundaries", "mod_boundaries_deps_heavy_deps_policy_rs::"),
];

// Module selectors end in `::`; leaf selectors must match the complete test name.
const SSOT: &[(&str, &str)] = &[
    ("contracts", "mod_contracts_tooling_policies_rs::policy__contracts__policies__policy_test_names_are_consistent"),
    ("contracts", "mod_contracts_tooling_registry_ssot_completeness_policy_rs::policy__contracts__registry_ssot_completeness_policy__supported_stages_and_tools_are_complete"),
    ("contracts", "mod_contracts_tooling_tool_registry_stage_domain_policy_rs::policy__contracts__tool_registry_stage_domain_policy__production_tool_bindings_are_explicit_and_domain_consistent"),
];
const REGISTRY: &[(&str, &str)] = &[
    ("contracts", "mod_contracts_tooling_tool_registry_reproducibility_policy_rs::policy__contracts__tool_registry_reproducibility_policy__production_registry_is_pinned_and_non_floating"),
    ("contracts", "mod_contracts_tooling_tool_registry_reproducibility_policy_rs::policy__contracts__tool_registry_reproducibility_policy__profiles_only_use_valid_production_tools"),
];
const RELEASE: &[(&str, &str)] = &[
    ("contracts", "mod_contracts_tooling_tool_registry_reproducibility_policy_rs::policy__contracts__tool_registry_reproducibility_policy__profiles_release_readiness_gate"),
    ("contracts", "mod_contracts_tooling_tool_registry_reproducibility_policy_rs::policy__contracts__tool_registry_reproducibility_policy__reference_adna_profile_uses_production_tools_only"),
];

fn arguments(operation: &str, groups: &[(&str, &str)]) -> Vec<String> {
    let mut args =
        ["nextest", operation, "--locked", "-p", "bijux-dna-policies"].map(str::to_string).to_vec();
    let binaries =
        groups.iter().map(|(binary, _)| *binary).collect::<std::collections::BTreeSet<_>>();
    for binary in binaries {
        args.extend(["--test".to_string(), binary.to_string()]);
    }
    if operation == "list" {
        args.extend(["--message-format", "json"].map(str::to_string));
    } else {
        args.extend(["--no-tests", "fail", "--no-fail-fast"].map(str::to_string));
    }
    args.extend(groups.iter().map(|(_, prefix)| (*prefix).to_string()));
    args
}

fn selected_coverage(inventory: &Value, groups: &[(&str, &str)]) -> Result<Vec<usize>> {
    let suites = inventory["rust-suites"].as_object().context("nextest list lacks rust-suites")?;
    let mut counts = vec![0; groups.len()];
    for suite in suites.values() {
        let binary = suite["binary-name"].as_str().context("nextest suite lacks binary-name")?;
        let cases = suite["testcases"].as_object().context("nextest suite lacks testcases")?;
        for (name, case) in cases {
            let status = case["filter-match"]["status"]
                .as_str()
                .context("nextest testcase lacks filter-match status")?;
            if status != "matches" {
                continue;
            }
            let group = groups
                .iter()
                .position(|(expected_binary, prefix)| {
                    binary == *expected_binary
                        && if prefix.ends_with("::") {
                            name.starts_with(prefix)
                        } else {
                            name == prefix
                        }
                })
                .with_context(|| format!("unexpected policy selection {binary}::{name}"))?;
            ensure!(
                case["ignored"].as_bool() == Some(false),
                "policy responsibility cannot be ignored: {binary}::{name}"
            );
            counts[group] += 1;
        }
    }
    for ((binary, prefix), count) in groups.iter().zip(&counts) {
        ensure!(*count > 0, "policy selected no executable tests for {binary}::{prefix}");
    }
    Ok(counts)
}

pub(super) fn run(
    workspace: &Workspace,
    envs: &[(String, String)],
    suite: &str,
) -> Result<OpsCommandOutcome> {
    let groups = match suite {
        "policy-fast" => GROUPS,
        "ssot-policy-fast" => SSOT,
        "registry-lint" => REGISTRY,
        "release-readiness" => RELEASE,
        _ => anyhow::bail!("unknown policy suite {suite}"),
    };
    if suite != "policy-fast" {
        return run_leaf_suite(workspace, envs, suite, groups);
    }
    let listing = run_program_with_env(workspace, "cargo", &arguments("list", groups), envs)?;
    if !listing.is_success() {
        return Ok(listing);
    }
    let inventory: Value = serde_json::from_str(&listing.stdout).context("parse nextest list")?;
    let counts = selected_coverage(&inventory, groups)?;
    write_json_pretty(
        &workspace.path(&format!("artifacts/ci/{suite}/inventory.json")),
        &inventory,
    )?;
    let mut outcome = run_program_with_env(workspace, "cargo", &arguments("run", groups), envs)?;
    outcome.stdout.insert_str(
        0,
        &format!(
            "{suite} selected {} executable tests across {} required policy selectors\n",
            counts.iter().sum::<usize>(),
            counts.len()
        ),
    );
    Ok(outcome)
}

fn harness_cases(listing: &str) -> Result<std::collections::BTreeSet<String>> {
    let mut names = std::collections::BTreeSet::new();
    for line in listing.lines() {
        if let Some(name) = line.strip_suffix(": test") {
            ensure!(names.insert(name.to_string()), "test harness repeated case {name}");
        }
    }
    Ok(names)
}

fn policy_test_executable(messages: &str) -> Result<String> {
    let mut executables = std::collections::BTreeSet::new();
    for line in messages.lines().filter(|line| !line.trim().is_empty()) {
        let message: Value = serde_json::from_str(line).context("parse Cargo build message")?;
        if message["reason"] == "compiler-artifact"
            && message["target"]["name"] == "contracts"
            && message["target"]["kind"] == serde_json::json!(["test"])
        {
            if let Some(executable) = message["executable"].as_str() {
                executables.insert(executable.to_string());
            }
        }
    }
    ensure!(executables.len() == 1, "Cargo must produce exactly one policy contracts executable");
    executables.into_iter().next().context("missing policy contracts executable")
}

fn run_leaf_suite(
    workspace: &Workspace,
    envs: &[(String, String)],
    suite: &str,
    groups: &[(&str, &str)],
) -> Result<OpsCommandOutcome> {
    let build_args = [
        "test",
        "--locked",
        "-p",
        "bijux-dna-policies",
        "--test",
        "contracts",
        "--no-run",
        "--message-format=json",
    ]
    .map(str::to_string);
    let build = run_program_with_env(workspace, "cargo", &build_args, envs)?;
    if !build.is_success() {
        return Ok(build);
    }
    let executable = policy_test_executable(&build.stdout)?;
    let test_workspace = Workspace { root: workspace.path("crates/bijux-dna-policies") };
    let mut arguments = vec!["--exact".to_string(), "--nocapture".to_string()];
    arguments.extend(groups.iter().map(|(_, name)| (*name).to_string()));
    let mut list_args = arguments.clone();
    list_args.extend(["--list", "--format", "terse"].map(str::to_string));
    let listing = run_program_with_env(&test_workspace, &executable, &list_args, envs)?;
    if !listing.is_success() {
        return Ok(listing);
    }
    let names = harness_cases(&listing.stdout)?;
    list_args.push("--ignored".to_string());
    let ignored_listing = run_program_with_env(&test_workspace, &executable, &list_args, envs)?;
    if !ignored_listing.is_success() {
        return Ok(ignored_listing);
    }
    let ignored = harness_cases(&ignored_listing.stdout)?;
    let cases = names
        .iter()
        .map(|name| {
            (
                name.clone(),
                serde_json::json!({
                    "ignored": ignored.contains(name), "filter-match": {"status": "matches"}
                }),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    // Normalize native harness lists into the same selection contract as nextest.
    let inventory = serde_json::json!({
        "executor": "rust-test-harness",
        "build_argv": build_args,
        "execution_argv": arguments,
        "listing": listing.stdout,
        "ignored_listing": ignored_listing.stdout,
        "rust-suites": {"policies::contracts": {
            "binary-name": "contracts", "binary-path": executable,
            "cwd": test_workspace.root, "testcases": cases
        }}
    });
    let counts = selected_coverage(&inventory, groups)?;
    write_json_pretty(
        &workspace.path(&format!("artifacts/ci/{suite}/inventory.json")),
        &inventory,
    )?;
    let mut outcome = run_program_with_env(&test_workspace, &executable, &arguments, envs)?;
    outcome.stdout.insert_str(
        0,
        &format!(
            "{suite} selected {} executable tests across {} required policy selectors\n",
            counts.iter().sum::<usize>(),
            counts.len()
        ),
    );
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn complete_inventory() -> Value {
        let mut inventory = json!({"rust-suites": {
            "policies::boundaries": {"binary-name": "boundaries", "testcases": {}},
            "policies::contracts": {"binary-name": "contracts", "testcases": {}}
        }});
        for (binary, prefix) in GROUPS {
            inventory["rust-suites"][format!("policies::{binary}")]["testcases"]
                [format!("{prefix}admitted")] =
                json!({"ignored": false, "filter-match": {"status": "matches"}});
        }
        inventory
    }

    #[test]
    fn retains_all_groups_and_ignores_unselected_cases() -> Result<()> {
        let mut inventory = complete_inventory();
        inventory["rust-suites"]["policies::contracts"]["testcases"]["unrelated"] =
            json!({"ignored": false, "filter-match": {"status": "mismatch"}});
        assert_eq!(selected_coverage(&inventory, GROUPS)?, vec![1; 7]);
        Ok(())
    }

    #[test]
    fn refuses_missing_groups_including_partial_selection() -> Result<()> {
        for (binary, prefix) in GROUPS {
            let mut inventory = complete_inventory();
            inventory["rust-suites"][format!("policies::{binary}")]["testcases"]
                .as_object_mut()
                .context("testcases")?
                .remove(&format!("{prefix}admitted"));
            assert!(selected_coverage(&inventory, GROUPS).is_err(), "missing {prefix}");
        }
        assert!(selected_coverage(&json!({"rust-suites": {}}), GROUPS).is_err());
        Ok(())
    }

    #[test]
    fn refuses_ignored_or_unexpected_selected_tests_and_malformed_inventory() {
        let mut inventory = complete_inventory();
        let name = format!("{}admitted", GROUPS[0].1);
        inventory["rust-suites"]["policies::boundaries"]["testcases"][name]["ignored"] =
            json!(true);
        assert!(selected_coverage(&inventory, GROUPS).is_err());
        let mut inventory = complete_inventory();
        inventory["rust-suites"]["policies::contracts"]["testcases"]["unrelated"] =
            json!({"ignored": false, "filter-match": {"status": "matches"}});
        assert!(selected_coverage(&inventory, GROUPS).is_err());
        assert!(selected_coverage(&json!({}), GROUPS).is_err());
    }
    #[test]
    fn leaf_suites_require_each_exact_case_and_reject_renamed_or_ignored_cases() -> Result<()> {
        for groups in [SSOT, REGISTRY, RELEASE] {
            let mut inventory = json!({"rust-suites": {"policies::contracts": {"binary-name": "contracts", "testcases": {}}}});
            for (_, name) in groups {
                inventory["rust-suites"]["policies::contracts"]["testcases"][*name] =
                    json!({"ignored": false, "filter-match": {"status": "matches"}});
            }
            assert_eq!(selected_coverage(&inventory, groups)?, vec![1; groups.len()]);
            for (_, name) in groups {
                let mut missing = inventory.clone();
                let cases = missing["rust-suites"]["policies::contracts"]["testcases"]
                    .as_object_mut()
                    .context("cases")?;
                let case = cases.remove(*name).context("required case")?;
                assert!(selected_coverage(&missing, groups).is_err());
                missing["rust-suites"]["policies::contracts"]["testcases"]
                    [format!("{name}_renamed")] = case;
                assert!(selected_coverage(&missing, groups).is_err());
                let mut ignored = inventory.clone();
                ignored["rust-suites"]["policies::contracts"]["testcases"][*name]["ignored"] =
                    json!(true);
                assert!(selected_coverage(&ignored, groups).is_err());
            }
        }
        Ok(())
    }
    #[test]
    fn cargo_executable_selection_rejects_missing_ambiguous_or_malformed_messages() -> Result<()> {
        let artifact = serde_json::json!({"reason": "compiler-artifact", "target": {"name": "contracts", "kind": ["test"]}, "executable": "/artifacts/contracts"}).to_string();
        assert_eq!(policy_test_executable(&artifact)?, "/artifacts/contracts");
        assert!(policy_test_executable("{}").is_err());
        assert!(policy_test_executable("not JSON").is_err());
        let other = artifact.replace("/artifacts/contracts", "/artifacts/other");
        assert!(policy_test_executable(&format!("{artifact}\n{other}")).is_err());
        Ok(())
    }

    #[test]
    fn harness_listing_preserves_qualified_names_and_rejects_duplicates() -> Result<()> {
        assert_eq!(
            harness_cases("owner::case: test\nother::case: test\n\n2 tests, 0 benchmarks\n")?,
            ["owner::case".to_string(), "other::case".to_string()].into_iter().collect()
        );
        assert!(harness_cases("owner::case: test\nowner::case: test\n").is_err());
        assert!(harness_cases("0 tests, 0 benchmarks\n")?.is_empty());
        Ok(())
    }
}
