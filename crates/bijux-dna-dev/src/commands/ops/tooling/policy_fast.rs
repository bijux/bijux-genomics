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

fn arguments(operation: &str) -> Vec<String> {
    let mut args = [
        "nextest",
        operation,
        "--locked",
        "-p",
        "bijux-dna-policies",
        "--test",
        "boundaries",
        "--test",
        "contracts",
    ]
    .map(str::to_string)
    .to_vec();
    if operation == "list" {
        args.extend(["--message-format", "json"].map(str::to_string));
    } else {
        args.extend(["--no-tests", "fail", "--no-fail-fast"].map(str::to_string));
    }
    args.extend(GROUPS.iter().map(|(_, prefix)| (*prefix).to_string()));
    args
}

fn selected_coverage(inventory: &Value) -> Result<Vec<usize>> {
    let suites = inventory["rust-suites"].as_object().context("nextest list lacks rust-suites")?;
    let mut counts = vec![0; GROUPS.len()];
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
            let group = GROUPS
                .iter()
                .position(|(expected_binary, prefix)| {
                    binary == *expected_binary && name.starts_with(prefix)
                })
                .with_context(|| format!("unexpected policy-fast selection {binary}::{name}"))?;
            ensure!(
                case["ignored"].as_bool() == Some(false),
                "policy-fast responsibility cannot be ignored: {binary}::{name}"
            );
            counts[group] += 1;
        }
    }
    for ((binary, prefix), count) in GROUPS.iter().zip(&counts) {
        ensure!(*count > 0, "policy-fast selected no executable tests for {binary}::{prefix}");
    }
    Ok(counts)
}

pub(super) fn run(workspace: &Workspace, envs: &[(String, String)]) -> Result<OpsCommandOutcome> {
    let listing = run_program_with_env(workspace, "cargo", &arguments("list"), envs)?;
    if !listing.is_success() {
        return Ok(listing);
    }
    let inventory: Value = serde_json::from_str(&listing.stdout).context("parse nextest list")?;
    let counts = selected_coverage(&inventory)?;
    write_json_pretty(&workspace.path("artifacts/ci/policy-fast/inventory.json"), &inventory)?;
    let mut outcome = run_program_with_env(workspace, "cargo", &arguments("run"), envs)?;
    outcome.stdout.insert_str(
        0,
        &format!(
            "policy-fast selected {} executable tests across {} required policy groups\n",
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
        assert_eq!(selected_coverage(&inventory)?, vec![1; 7]);
        Ok(())
    }

    #[test]
    fn refuses_missing_groups_including_partial_selection() {
        for (binary, prefix) in GROUPS {
            let mut inventory = complete_inventory();
            inventory["rust-suites"][format!("policies::{binary}")]["testcases"]
                .as_object_mut()
                .expect("testcases")
                .remove(&format!("{prefix}admitted"));
            assert!(selected_coverage(&inventory).is_err(), "missing {prefix}");
        }
        assert!(selected_coverage(&json!({"rust-suites": {}})).is_err());
    }

    #[test]
    fn refuses_ignored_or_unexpected_selected_tests_and_malformed_inventory() {
        let mut inventory = complete_inventory();
        let name = format!("{}admitted", GROUPS[0].1);
        inventory["rust-suites"]["policies::boundaries"]["testcases"][name]["ignored"] =
            json!(true);
        assert!(selected_coverage(&inventory).is_err());
        let mut inventory = complete_inventory();
        inventory["rust-suites"]["policies::contracts"]["testcases"]["unrelated"] =
            json!({"ignored": false, "filter-match": {"status": "matches"}});
        assert!(selected_coverage(&inventory).is_err());
        assert!(selected_coverage(&json!({})).is_err());
    }
}
