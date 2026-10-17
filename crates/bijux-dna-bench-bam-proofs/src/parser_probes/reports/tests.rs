#![cfg(test)]

use super::{parse_report, parse_report_contents, BamReportKind};
use anyhow::{Context, Result};
use serde_json::Value;

const VALIDATION: &str = r#"{"schema_version":"bijux.bam.validate.v1","stage_id":"bam.validate","input_bam":"original.bam","flagstat":{},"validation_report_present":true}"#;

#[test]
fn captured_admission_keeps_bytes_after_the_source_path_changes() -> Result<()> {
    let owner = tempfile::tempdir()?;
    let path = owner.path().join("report.json");
    for (kind, raw) in [
        (BamReportKind::ValidationSummary, VALIDATION),
        (BamReportKind::ContaminationJson, r#"{"method":"authentiCT","estimate":0.12}"#),
        (BamReportKind::SexJson, r#"{"method":"rxy","x_to_y_ratio":0.4,"confidence":0.8}"#),
        (BamReportKind::GenotypingJson, r#"{"producer":"original-producer"}"#),
    ] {
        std::fs::write(&path, raw)?;
        let original = parse_report(&path, kind)?;
        let captured = std::fs::read_to_string(&path)?;
        std::fs::write(&path, r#"{"replacement":true}"#)?;
        assert_eq!(parse_report_contents(&path, &captured, kind)?, original);
        assert_eq!(original, serde_json::from_str::<Value>(raw)?);
        std::fs::remove_file(&path)?;
        assert_eq!(parse_report_contents(&path, &captured, kind)?, original);
        assert!(parse_report(&path, kind).is_err(), "a new request must still acquire its path");
    }
    Ok(())
}

#[test]
fn captured_typed_admission_rejects_duplicates_before_projection() -> Result<()> {
    let owner = tempfile::tempdir()?;
    let path = owner.path().join("report.json");
    let duplicate = r#"{"schema_version":"first","schema_version":"second","stage_id":"bam.validate","input_bam":"original.bam","flagstat":{},"validation_report_present":true}"#;
    std::fs::write(&path, duplicate)?;
    let captured = std::fs::read_to_string(&path)?;
    std::fs::write(&path, VALIDATION)?;
    assert!(parse_report(&path, BamReportKind::ValidationSummary).is_ok());
    let error = parse_report_contents(&path, &captured, BamReportKind::ValidationSummary)
        .err()
        .context("duplicate typed field accepted")?;
    let diagnostic = format!("{error:#}");
    assert!(diagnostic.contains("duplicate field `schema_version`"));
    assert!(diagnostic.contains(&format!("parse {}", path.display())));
    Ok(())
}
