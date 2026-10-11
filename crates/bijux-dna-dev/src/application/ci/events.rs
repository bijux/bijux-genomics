use std::collections::BTreeMap;

use anyhow::{bail, ensure, Context, Result};
use serde::Deserialize;

use super::inventory::{ExpectedTestInventory, SuiteIdentity, TestIdentity};

#[derive(Deserialize)]
struct WireRecord {
    #[serde(rename = "type")]
    event_type: String,
    event: String,
    name: Option<String>,
    nextest: Option<SuiteIdentity>,
    test_count: Option<usize>,
    passed: Option<usize>,
    failed: Option<usize>,
    ignored: Option<usize>,
    measured: Option<usize>,
}

struct SuiteState {
    declared_count: usize,
    started: BTreeMap<String, Option<bool>>,
    closed: bool,
}

pub(super) fn admit_shard_events(
    inventory: &ExpectedTestInventory,
    bytes: &[u8],
) -> Result<BTreeMap<TestIdentity, bool>> {
    let text = std::str::from_utf8(bytes).context("nextest events are not UTF-8")?;
    let mut suites = BTreeMap::<SuiteIdentity, SuiteState>::new();
    let mut observed = BTreeMap::new();
    for (line_number, line) in text.lines().enumerate().filter(|(_, line)| !line.trim().is_empty())
    {
        let event: WireRecord = serde_json::from_str(line)
            .with_context(|| format!("parse nextest event line {}", line_number + 1))?;
        match event.event_type.as_str() {
            "suite" => {
                let identity = event.nextest.context("suite event has no nextest identity")?;
                let facts = inventory.suites.get(&identity).context("unexpected suite identity")?;
                match event.event.as_str() {
                    "started" => {
                        let state = SuiteState {
                            declared_count: event
                                .test_count
                                .context("suite start has no test count")?,
                            started: BTreeMap::new(),
                            closed: false,
                        };
                        ensure!(
                            state.declared_count <= facts.case_count,
                            "suite declares more cases than its inventory"
                        );
                        if let Some(previous) = suites.get(&identity) {
                            // Nextest 0.9.100 can close and reopen a suite when ignored tests run.
                            // Reopened blocks never reset the global observed-case identities.
                            ensure!(
                                previous.closed
                                    && facts.ignored_count > 0
                                    && previous.declared_count == state.declared_count,
                                "unexpected repeated suite start"
                            );
                        }
                        suites.insert(identity, state);
                    }
                    "ok" | "failed" => {
                        let state =
                            suites.get_mut(&identity).context("suite finished without starting")?;
                        ensure!(!state.closed, "suite finished more than once");
                        ensure!(
                            state.started.len() <= state.declared_count
                                && (facts.ignored_count > 0
                                    || state.started.len() == state.declared_count)
                                && state.started.values().all(Option::is_some),
                            "suite closed with missing test events"
                        );
                        let passed =
                            state.started.values().filter(|value| **value == Some(true)).count();
                        let failed = state.started.len() - passed;
                        ensure!(
                            event.passed == Some(passed)
                                && event.failed == Some(failed)
                                && event.ignored.is_some_and(|count| count <= facts.ignored_count)
                                && event.measured == Some(0),
                            "suite counters disagree with observed results"
                        );
                        ensure!(
                            (event.event == "ok") == (failed == 0),
                            "suite verdict disagrees with results"
                        );
                        state.closed = true;
                    }
                    other => bail!("unsupported suite event: {other}"),
                }
            }
            "test" => {
                let name = event.name.context("test event has no name")?;
                let (identity, suite) =
                    inventory.cases.get(&name).context("unexpected test identity")?;
                let state = suites.get_mut(suite).context("test event has no started suite")?;
                ensure!(!state.closed, "test event occurs after suite completion");
                match event.event.as_str() {
                    "started" => ensure!(
                        state.started.insert(name, None).is_none(),
                        "test started more than once"
                    ),
                    "ok" | "failed" => {
                        let result = state
                            .started
                            .get_mut(&name)
                            .context("test finished without starting")?;
                        ensure!(result.is_none(), "test finished more than once");
                        let passed = event.event == "ok";
                        *result = Some(passed);
                        ensure!(
                            observed.insert(identity.clone(), passed).is_none(),
                            "duplicate test result"
                        );
                    }
                    other => bail!("unsupported test event under run-ignored=all: {other}"),
                }
            }
            other => bail!("unsupported nextest event type: {other}"),
        }
    }
    ensure!(!observed.is_empty(), "shard produced no observed test results");
    ensure!(suites.values().all(|suite| suite.closed), "shard has unfinished suites");
    Ok(observed)
}
