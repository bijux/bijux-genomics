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
    // Lifetime membership is bounded by the inventory, while each frame has its own slots.
    started: BTreeMap<String, Option<bool>>,
    started_in_frame: usize,
    completed_in_frame: Vec<bool>,
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
                            started_in_frame: 0,
                            completed_in_frame: Vec::new(),
                            closed: false,
                        };
                        ensure!(
                            state.declared_count <= facts.case_count,
                            "suite declares more cases than its inventory"
                        );
                        if let Some(previous) = suites.get_mut(&identity) {
                            // The pinned reporter decrements an ordinary-only running count for
                            // ignored completions too. Its frame can close while a test is live;
                            // a later frame contains that terminal event without another start.
                            ensure!(
                                previous.closed
                                    && facts.ignored_count > 0
                                    && previous.declared_count == state.declared_count,
                                "unexpected repeated suite start"
                            );
                            previous.started_in_frame = 0;
                            previous.completed_in_frame.clear();
                            previous.closed = false;
                        } else {
                            suites.insert(identity, state);
                        }
                    }
                    "ok" | "failed" => {
                        let state =
                            suites.get_mut(&identity).context("suite finished without starting")?;
                        ensure!(!state.closed, "suite finished more than once");
                        ensure!(
                            state.started.len() <= facts.case_count
                                && state.started_in_frame <= state.declared_count
                                && state.completed_in_frame.len() <= state.declared_count
                                && (facts.ignored_count > 0
                                    || (state.started_in_frame == state.declared_count
                                        && state.completed_in_frame.len() == state.declared_count
                                        && state.started.values().all(Option::is_some))),
                            "suite closed with missing test events"
                        );
                        let passed =
                            state.completed_in_frame.iter().filter(|passed| **passed).count();
                        let failed = state.completed_in_frame.len() - passed;
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
                let facts = inventory.suites.get(suite).context("unexpected suite identity")?;
                let state = suites.get_mut(suite).context("test event has no started suite")?;
                ensure!(!state.closed, "test event occurs after suite completion");
                match event.event.as_str() {
                    "started" => {
                        ensure!(
                            state.started.insert(name, None).is_none(),
                            "test started more than once"
                        );
                        state.started_in_frame = state
                            .started_in_frame
                            .checked_add(1)
                            .context("frame start count overflow")?;
                        ensure!(
                            state.started.len() <= facts.case_count,
                            "suite lifetime starts exceed its inventory"
                        );
                        ensure!(
                            state.started_in_frame <= state.declared_count,
                            "frame starts exceed its declared count"
                        );
                    }
                    "ok" | "failed" => {
                        let result = state
                            .started
                            .get_mut(&name)
                            .context("test finished without starting")?;
                        ensure!(result.is_none(), "test finished more than once");
                        let passed = event.event == "ok";
                        *result = Some(passed);
                        state.completed_in_frame.push(passed);
                        ensure!(
                            state.completed_in_frame.len() <= state.declared_count,
                            "frame completions exceed its declared count"
                        );
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
    ensure!(
        suites.values().all(|suite| suite.started.values().all(Option::is_some)),
        "shard has unfinished test cases across reporter frames"
    );
    Ok(observed)
}

#[cfg(test)]
mod pinned_frames;
