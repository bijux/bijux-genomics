use anyhow::Result;
use bijux_analyze::BenchmarkFailure;
use bijux_core::prelude::errors::{ErrorCategory, RawFailure};
use std::path::PathBuf;

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-analyze__{group}__{name}")
}

fn assert_snapshot(name: &str, failure: &BenchmarkFailure) -> Result<()> {
    let json = serde_json::to_value(failure)?;
    let name = snapshot_name("schemas", name);
    let mut settings = insta::Settings::new();
    settings.set_snapshot_path(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("snapshots"),
    );
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        insta::assert_json_snapshot!(name, json);
    });
    Ok(())
}

/// Snapshot locks failure hint for adapter issues.
#[test]
fn failure_hint_adapter_snapshot() -> Result<()> {
    let failure = bijux_analyze::failure::classify_raw_failure(&RawFailure {
        stage: "fastq.trim".to_string(),
        tool: "fastp".to_string(),
        reason: "adapter preset missing".to_string(),
        category: ErrorCategory::ContractError,
    });
    assert_snapshot("failure_hint_adapter", &failure)
}

/// Snapshot locks failure hint for timeout issues.
#[test]
fn failure_hint_timeout_snapshot() -> Result<()> {
    let failure = bijux_analyze::failure::classify_raw_failure(&RawFailure {
        stage: "fastq.trim".to_string(),
        tool: "fastp".to_string(),
        reason: "timeout while running tool".to_string(),
        category: ErrorCategory::ToolError,
    });
    assert_snapshot("failure_hint_timeout", &failure)
}

/// Snapshot locks failure hint for invalid input issues.
#[test]
fn failure_hint_invalid_snapshot() -> Result<()> {
    let failure = bijux_analyze::failure::classify_raw_failure(&RawFailure {
        stage: "fastq.validate_pre".to_string(),
        tool: "fastqvalidator".to_string(),
        reason: "invalid fastq records".to_string(),
        category: ErrorCategory::PlanError,
    });
    assert_snapshot("failure_hint_invalid", &failure)
}
