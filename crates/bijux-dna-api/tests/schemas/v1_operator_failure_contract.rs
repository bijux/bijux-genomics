use anyhow::anyhow;
use bijux_dna_api::v1::api::run::{classify_operator_failure, CategorizedError, ErrorCategory};

#[test]
fn operator_failure_contract_is_stable() -> anyhow::Result<()> {
    let err = anyhow!(CategorizedError::new(
        ErrorCategory::ToolError,
        "tool failed with exit code 1",
    ));
    let failure = classify_operator_failure(&err);
    assert_eq!(failure.schema_version, "bijux.operator_failure.v1");
    assert_eq!(failure.category, ErrorCategory::ToolError);
    assert!(!failure.hints.is_empty());
    let json = serde_json::to_value(&failure)?;
    assert_eq!(
        json.get("schema_version").and_then(|v| v.as_str()),
        Some("bijux.operator_failure.v1")
    );
    assert!(json.get("hints").and_then(|v| v.as_array()).is_some());
    Ok(())
}
