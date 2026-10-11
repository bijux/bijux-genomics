#![cfg(test)]

use super::*;
use anyhow::{Context, Result};
use serde_json::json;
use std::fs;

#[test]
fn json_read_and_parse_errors_identify_the_actual_source() -> Result<()> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("actual.json");
    let missing = read_json(&path).err().context("missing source must fail")?;
    assert_eq!(missing.to_string(), format!("read {}", path.display()));
    fs::write(&path, "{broken")?;
    let invalid = read_json(&path).err().context("malformed source must fail")?;
    assert_eq!(invalid.to_string(), format!("parse {}", path.display()));
    Ok(())
}

#[test]
fn document_addresses_are_admitted_before_any_document_read() -> Result<()> {
    let root = tempfile::tempdir()?;
    let report = root.path().join("proof.json");
    let error = required_json_documents(
        root.path(),
        &json!({"first": "absent.json"}),
        &report,
        ["first", "second"],
    )
    .err()
    .context("missing address must fail before absent document")?;
    assert_eq!(error.to_string(), format!("{} is missing string `second`", report.display()));
    fs::write(root.path().join("first.json"), r#"{"id":1}"#)?;
    fs::write(root.path().join("second.json"), r#"{"id":2}"#)?;
    let documents = required_json_documents(
        root.path(),
        &json!({"first": "first.json", "second": "second.json"}),
        &report,
        ["second", "first"],
    )?;
    assert_eq!(documents, [json!({"id":2}), json!({"id":1})]);
    Ok(())
}

#[test]
fn proof_artifacts_retain_report_identity_order_and_repeated_addresses() -> Result<()> {
    let root = tempfile::tempdir()?;
    let report = root.path().join("actual-proof.json");
    fs::write(root.path().join("a.json"), "{}")?;
    fs::write(root.path().join("b.json"), "{}")?;
    let paths = required_proof_artifacts(
        root.path(),
        &json!({"a":"a.json","b":"b.json"}),
        &report,
        &["b", "a", "b"],
        crate::readiness::damage_authenticity::input::ensure_repo_relative_file,
    )?;
    assert_eq!(paths, ["actual-proof.json", "b.json", "a.json", "b.json"]);
    Ok(())
}

#[test]
fn retained_file_admission_distinguishes_files_from_legacy_existing_paths() -> Result<()> {
    let root = tempfile::tempdir()?;
    fs::create_dir(root.path().join("directory"))?;
    let strict = crate::readiness::damage_authenticity::input::ensure_repo_relative_file(
        root.path(),
        "directory",
    )
    .err()
    .context("smoke artifact directory must fail")?;
    assert!(strict.to_string().contains("governed local-smoke artifact is missing"));
    let legacy = crate::readiness::recalibration_genotyping::input::ensure_repo_relative_file(
        root.path(),
        "directory",
    )?;
    assert_eq!(legacy, "directory");
    Ok(())
}
