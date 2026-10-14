use std::collections::{BTreeMap, BTreeSet};

use anyhow::{ensure, Context, Result};
use serde_json::Value;

use super::super::{CoverageAttestation, ExpectedTestInventory, ShardDiscoveryPlan};
use super::CaseOutcome;

pub(crate) struct NextestCaseEvidence {
    pub expected: Vec<String>,
    pub outcomes: Vec<CaseOutcome>,
    pub errors: Vec<String>,
}

pub(crate) fn nextest_membership(
    inventory_bytes: &[u8],
    streams: &[(String, String, Vec<u8>)],
) -> Result<NextestCaseEvidence> {
    let inventory = ExpectedTestInventory::parse(inventory_bytes)?;
    let plans = streams
        .iter()
        .map(|(_, partition, _)| {
            ShardDiscoveryPlan::from_inventory(&inventory, partition, "0.9.100")
        })
        .collect::<Result<Vec<_>>>()?;
    let coverage = CoverageAttestation::reconcile_planned(
        &inventory,
        streams
            .iter()
            .zip(&plans)
            .map(|((id, _, bytes), plan)| (id.as_str(), bytes.as_slice(), plan)),
    );
    let mut errors = Vec::new();
    if !coverage.complete || !coverage.selected_shards_valid {
        errors.push(format!(
            "native Nextest case union failed: {}",
            serde_json::to_string(&coverage)?
        ));
    }
    let expected = inventory
        .cases
        .values()
        .map(|(identity, _)| format!("{}${}", identity.binary_id, identity.test_name))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut terminal = BTreeMap::new();
    for (_, _, bytes) in streams {
        for line in bytes.split(|byte| *byte == b'\n').filter(|line| !line.is_empty()) {
            let Ok(event) = serde_json::from_slice::<Value>(line) else {
                continue;
            };
            if event["type"] != "test" || event["event"] == "started" {
                continue;
            }
            let Some(wire) = event["name"].as_str() else {
                continue;
            };
            let id = inventory.cases.get(wire).map_or_else(
                || format!("unexpected:${wire}"),
                |(identity, _)| format!("{}${}", identity.binary_id, identity.test_name),
            );
            let outcome = if event["event"] == "ok" {
                "passed"
            } else if event["event"] == "failed" {
                "failed"
            } else {
                "cancelled_or_skipped"
            };
            if terminal.insert(id.clone(), outcome.to_string()).is_some() {
                terminal.insert(id, "duplicate_terminal".into());
            }
        }
    }
    for id in &expected {
        terminal.entry(id.clone()).or_insert_with(|| "not_observed".into());
    }
    let outcomes = terminal.into_iter().map(|(id, outcome)| CaseOutcome { id, outcome }).collect();
    Ok(NextestCaseEvidence { expected, outcomes, errors })
}

pub(crate) fn doctest_targets(metadata: &Value) -> Result<Vec<String>> {
    let members = metadata["workspace_members"]
        .as_array()
        .context("workspace members absent")?
        .iter()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    let mut targets = BTreeSet::new();
    for package in metadata["packages"].as_array().context("workspace packages absent")? {
        if !members.contains(package["id"].as_str().context("package identity absent")?) {
            continue;
        }
        for target in package["targets"].as_array().context("package targets absent")? {
            if target["doctest"] == true
                && target["kind"].as_array().is_some_and(|kinds| {
                    kinds.iter().any(|kind| kind == "lib" || kind == "proc-macro")
                })
            {
                ensure!(
                    targets.insert(
                        target["name"].as_str().context("doctest target name absent")?.to_string()
                    ),
                    "ambiguous native doctest target identity"
                );
            }
        }
    }
    ensure!(!targets.is_empty(), "workspace doctest target universe is empty");
    Ok(targets.into_iter().collect())
}

fn observed_targets(stderr: &str, declared: &[String]) -> Result<Vec<String>> {
    let observed = stderr
        .lines()
        .filter_map(|line| line.trim().strip_prefix("Doc-tests ").map(str::to_string))
        .collect::<Vec<_>>();
    let actual = observed.iter().collect::<BTreeSet<_>>();
    ensure!(
        actual.len() == observed.len() && actual == declared.iter().collect(),
        "missing, duplicate or unexpected native doctest target"
    );
    Ok(observed)
}

pub(crate) fn doctest_inventory(
    declared: &[String],
    stdout: &str,
    stderr: &str,
) -> Result<Vec<String>> {
    let targets = observed_targets(stderr, declared)?;
    let mut blocks = Vec::new();
    let mut cases = Vec::new();
    for line in stdout.lines() {
        if let Some(name) = line.strip_suffix(": test") {
            cases.push(name.to_string());
        } else if let Some((tests, rest)) =
            line.split_once(" tests, ").or_else(|| line.split_once(" test, "))
        {
            if rest.ends_with(" benchmarks") {
                let count = tests.parse::<usize>()?;
                ensure!(
                    count == cases.len(),
                    "native doctest discovery count disagrees with cases"
                );
                blocks.push(std::mem::take(&mut cases));
            }
        }
    }
    ensure!(
        cases.is_empty() && blocks.len() == targets.len(),
        "truncated or missing native doctest discovery block"
    );
    let mut expected = Vec::new();
    for (target, cases) in targets.into_iter().zip(blocks) {
        expected.push(format!("{target}::@target"));
        expected.extend(cases.into_iter().map(|name| format!("{target}::{name}")));
    }
    expected.sort();
    ensure!(
        expected.iter().collect::<BTreeSet<_>>().len() == expected.len(),
        "duplicate native doctest discovery identity"
    );
    Ok(expected)
}

pub(crate) fn doctest_outcomes(
    declared: &[String],
    expected: &[String],
    stdout: &str,
    stderr: &str,
) -> Result<Vec<CaseOutcome>> {
    let targets = observed_targets(stderr, declared)?;
    let mut blocks = Vec::new();
    let mut cases = Vec::new();
    let mut running = None;
    for line in stdout.lines() {
        if let Some(count) = line
            .strip_prefix("running ")
            .and_then(|line| line.strip_suffix(" tests").or_else(|| line.strip_suffix(" test")))
        {
            ensure!(
                running.replace(count.parse::<usize>()?).is_none(),
                "nested native doctest result block"
            );
        } else if let Some((name, status)) =
            line.strip_prefix("test ").and_then(|line| line.rsplit_once(" ... "))
        {
            ensure!(running.is_some(), "native doctest result outside a block");
            cases.push((
                name.to_string(),
                if status == "ok" { "passed" } else { "failed" }.to_string(),
            ));
        } else if line.starts_with("test result:") {
            ensure!(
                running.take() == Some(cases.len()),
                "native doctest execution count disagrees with actual cases"
            );
            let passing = line.starts_with("test result: ok.");
            blocks.push((passing, std::mem::take(&mut cases)));
        }
    }
    ensure!(
        running.is_none() && cases.is_empty() && blocks.len() == targets.len(),
        "truncated native doctest execution"
    );
    let mut outcomes = Vec::new();
    for (target, (passing, cases)) in targets.into_iter().zip(blocks) {
        outcomes.push(CaseOutcome {
            id: format!("{target}::@target"),
            outcome: if passing { "passed" } else { "failed" }.into(),
        });
        outcomes.extend(
            cases
                .into_iter()
                .map(|(name, outcome)| CaseOutcome { id: format!("{target}::{name}"), outcome }),
        );
    }
    outcomes.sort_by(|left, right| left.id.cmp(&right.id));
    ensure!(
        outcomes.len() == expected.len()
            && outcomes.iter().map(|case| &case.id).collect::<Vec<_>>()
                == expected.iter().collect::<Vec<_>>(),
        "native doctest discovery/execution membership changed"
    );
    Ok(outcomes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_and_truncated_nextest_keep_discovered_membership_and_terminal_cases() -> Result<()> {
        let inventory = serde_json::to_vec(
            &serde_json::json!({"test-count":1,"rust-suites":{"owner::contracts":{
            "package-name":"owner","binary-id":"owner::contracts","binary-name":"contracts","kind":"test","status":"listed",
            "testcases":{"retained":{"ignored":false,"filter-match":{"status":"matches"}}}}}}),
        )?;
        let events = [
            serde_json::json!({"type":"suite","event":"started","test_count":1,"nextest":{"crate":"owner","test_binary":"contracts","kind":"test"}}),
            serde_json::json!({"type":"test","event":"started","name":"owner::contracts$retained"}),
            serde_json::json!({"type":"test","event":"failed","name":"owner::contracts$retained"}),
            serde_json::json!({"type":"suite","event":"failed","passed":0,"failed":1,"ignored":0,"measured":0,"nextest":{"crate":"owner","test_binary":"contracts","kind":"test"}}),
        ];
        let stream =
            events.iter().map(Value::to_string).collect::<Vec<_>>().join("\n").into_bytes();
        let evidence =
            nextest_membership(&inventory, &[("full".into(), "hash:1/1".into(), stream)])?;
        assert_eq!(evidence.expected, vec!["owner::contracts$retained"]);
        assert_eq!(evidence.outcomes[0].id, evidence.expected[0]);
        assert_eq!(evidence.outcomes[0].outcome, "failed");
        assert!(!evidence.errors.is_empty());
        let truncated = nextest_membership(
            &inventory,
            &[("full".into(), "hash:1/1".into(), events[0].to_string().into_bytes())],
        )?;
        assert_eq!(truncated.expected, evidence.expected);
        assert_eq!(truncated.outcomes[0].outcome, "not_observed");
        assert!(!truncated.errors.is_empty());
        Ok(())
    }

    #[test]
    fn zero_example_target_and_actual_example_have_exact_native_membership() -> Result<()> {
        let targets = vec!["empty".into(), "examples".into()];
        let stderr = "   Doc-tests empty\n   Doc-tests examples\n";
        let discovered = doctest_inventory(
            &targets,
            "0 tests, 0 benchmarks\nsource.rs - example (line 7): test\n1 tests, 0 benchmarks\n",
            stderr,
        )?;
        let output = "running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\nrunning 1 test\ntest source.rs - example (line 7) ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n";
        let admitted = doctest_outcomes(&targets, &discovered, output, stderr)?;
        assert_eq!(admitted.len(), 3);
        assert!(admitted.iter().all(|case| case.outcome == "passed"));
        for (stdout, stderr) in [
            (output.replace("... ok", "... ignored"), stderr.to_string()),
            (output.to_string(), stderr.replace("examples", "empty")),
            (output.replace("running 1 test", "running 2 tests"), stderr.to_string()),
            (output.lines().take(4).collect::<Vec<_>>().join("\n"), stderr.to_string()),
        ] {
            assert!(
                doctest_outcomes(&targets, &discovered, &stdout, &stderr).is_err()
                    || doctest_outcomes(&targets, &discovered, &stdout, &stderr)?
                        .iter()
                        .any(|case| case.outcome != "passed")
            );
        }
        Ok(())
    }
}
