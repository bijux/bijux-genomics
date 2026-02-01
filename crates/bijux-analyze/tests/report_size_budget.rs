use std::fs;
use std::path::PathBuf;

#[test]
fn report_size_budget_is_bounded() -> anyhow::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let report_path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("run_report.json");
    let raw = fs::read_to_string(&report_path)?;
    let max_len = 250_000usize;
    assert!(
        raw.len() <= max_len,
        "report size {} exceeds max {}",
        raw.len(),
        max_len
    );

    let report: serde_json::Value = serde_json::from_str(&raw)?;
    let sections = report
        .get("sections")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow::anyhow!("missing sections"))?;
    let expected_sections = 23usize;
    assert_eq!(sections.len(), expected_sections, "section count changed");
    Ok(())
}
