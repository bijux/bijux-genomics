use bijux_core::primitives::RawFailure;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use bijux_analyze::classify_raw_failure;

fn snapshot_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(name)
}

fn assert_snapshot(name: &str, failure: &bijux_analyze::BenchmarkFailure) -> Result<()> {
    let rendered = serde_json::to_string_pretty(failure)?;
    let snapshot = fs::read_to_string(snapshot_path(name))?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}

#[test]
fn failure_hint_adapter_snapshot() -> Result<()> {
    let raw = RawFailure {
        stage: "fastq.trim".to_string(),
        tool: "fastp".to_string(),
        reason: "adapter preset missing".to_string(),
        category: bijux_core::primitives::errors::ErrorCategory::DataError,
    };
    let failure = classify_raw_failure(&raw);
    assert_snapshot("failure_hint_adapter.json", &failure)
}

#[test]
fn failure_hint_timeout_snapshot() -> Result<()> {
    let raw = RawFailure {
        stage: "fastq.trim".to_string(),
        tool: "fastp".to_string(),
        reason: "timeout while running tool".to_string(),
        category: bijux_core::primitives::errors::ErrorCategory::ToolError,
    };
    let failure = classify_raw_failure(&raw);
    assert_snapshot("failure_hint_timeout.json", &failure)
}

#[test]
fn failure_hint_invalid_snapshot() -> Result<()> {
    let raw = RawFailure {
        stage: "fastq.validate_pre".to_string(),
        tool: "fastqvalidator".to_string(),
        reason: "invalid fastq record".to_string(),
        category: bijux_core::primitives::errors::ErrorCategory::DataError,
    };
    let failure = classify_raw_failure(&raw);
    assert_snapshot("failure_hint_invalid.json", &failure)
}
