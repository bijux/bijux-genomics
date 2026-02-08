use anyhow::Result;
use bijux_analyze::BenchmarkFailure;
use bijux_testkit::snapshot_name;

fn assert_snapshot(name: &str, failure: &BenchmarkFailure) -> Result<()> {
    let json = serde_json::to_value(failure)?;
    let name = snapshot_name("schemas", name);
    insta::assert_json_snapshot!(name, json);
    Ok(())
}

/// Snapshot locks failure hint for adapter issues.
#[test]
fn failure_hint_adapter_snapshot() -> Result<()> {
    let failure = bijux_analyze::failure::fixture_adapter_failure();
    assert_snapshot("failure_hint_adapter", &failure)
}

/// Snapshot locks failure hint for timeout issues.
#[test]
fn failure_hint_timeout_snapshot() -> Result<()> {
    let failure = bijux_analyze::failure::fixture_timeout_failure();
    assert_snapshot("failure_hint_timeout", &failure)
}

/// Snapshot locks failure hint for invalid input issues.
#[test]
fn failure_hint_invalid_snapshot() -> Result<()> {
    let failure = bijux_analyze::failure::fixture_invalid_failure();
    assert_snapshot("failure_hint_invalid", &failure)
}
