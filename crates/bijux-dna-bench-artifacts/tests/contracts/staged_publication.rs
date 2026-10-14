//! Consuming publication preserves stable bytes and refuses unadmitted replacements.
use anyhow::Result;
use bijux_dna_bench_artifacts::{PublicationSchema, ReportOwner, StagedPublicationPlan};
use std::fs;
use std::path::Path;

const SCHEMA: PublicationSchema = PublicationSchema {
    inventory_name: "owned-artifacts.json",
    schema_version: "example.owned_artifacts.v1",
};
fn owner() -> ReportOwner<'static> {
    ReportOwner {
        report_name: "metrics.json",
        schema: "example.metrics.v1",
        stage: "vcf.qc",
        tool: "plink2",
    }
}
fn staged(path: &Path, payload: &[u8]) -> Result<()> {
    fs::create_dir_all(path.join("artifacts"))?;
    fs::write(path.join("artifacts/native.tsv"), payload)?;
    fs::write(path.join("metrics.json"), br#"{"schema_version":"example.metrics.v1","stage_id":"vcf.qc","tool_id":"plink2","science":{"values":[1,2]}}"#)?;
    Ok(())
}

#[test]
fn consumption_preserves_scientific_and_stage_result_bytes_with_stable_only_inventory() -> Result<()>
{
    let root = tempfile::tempdir()?;
    let source = root.path().join("staged");
    let destination = root.path().join("published");
    staged(&source, b"sample\tvalue\nsample-A\t1.25\n")?;
    // This protocol is already projected by the capability's stage-result owner.
    let stage_result = br#"{"output":"published/artifacts/native.tsv","native_command":["tool","--scientific-mode"]}"#;
    fs::write(source.join("stage-result.json"), stage_result)?;
    let before = ["metrics.json", "artifacts/native.tsv", "stage-result.json"]
        .into_iter()
        .map(|name| {
            Ok((name, fs::read(source.join(name))?, fs::metadata(source.join(name))?.permissions()))
        })
        .collect::<Result<Vec<_>>>()?;
    StagedPublicationPlan::admit(&source, &destination, &owner(), &SCHEMA)?.publish()?;
    for (name, bytes, permissions) in before {
        assert_eq!(fs::read(destination.join(name))?, bytes);
        assert_eq!(fs::metadata(destination.join(name))?.permissions(), permissions);
        assert!(!source.join(name).exists(), "source {name} must be consumed");
    }
    assert!(source.join("artifacts").is_dir());
    assert!(!destination.join("executions").exists());
    let inventory: serde_json::Value =
        serde_json::from_slice(&fs::read(destination.join(SCHEMA.inventory_name))?)?;
    assert_eq!(
        inventory,
        serde_json::json!({
            "schema_version": SCHEMA.schema_version, "report_schema": "example.metrics.v1",
            "stage_id": "vcf.qc", "tool_id": "plink2", "files": ["artifacts/native.tsv", "metrics.json", "stage-result.json"]
        })
    );
    Ok(())
}

#[test]
fn report_refusal_preserves_both_roots_before_any_consumption() -> Result<()> {
    let root = tempfile::tempdir()?;
    let destination = root.path().join("published");
    let original = root.path().join("original");
    staged(&original, b"original native")?;
    StagedPublicationPlan::admit(&original, &destination, &owner(), &SCHEMA)?.publish()?;
    let stable = ["metrics.json", "artifacts/native.tsv", SCHEMA.inventory_name]
        .into_iter()
        .map(|name| Ok((name, fs::read(destination.join(name))?)))
        .collect::<Result<Vec<_>>>()?;
    let source = root.path().join("malformed");
    staged(&source, b"replacement native")?;
    let invalid: [&[u8]; 8] = [
        b"{invalid", b"[]", b"null",
        br#"{"schema_version":"example.metrics.v1","stage_id":"vcf.qc"}"#,
        br#"{"schema_version":"example.metrics.v1","stage_id":0,"tool_id":"plink2"}"#,
        br#"{"schema_version":"example.metrics.v1","stage_id":"vcf.qc","tool_id":"foreign"}"#,
        br#"{"schema_version":"foreign","schema_version":"example.metrics.v1","stage_id":"vcf.qc","tool_id":"plink2"}"#,
        br#"{"schema_version":"example.metrics.v1","stage_id":"foreign","stage_id":"vcf.qc","tool_id":"plink2"}"#,
    ];
    for payload in invalid {
        fs::write(source.join("metrics.json"), payload)?;
        assert!(StagedPublicationPlan::admit(&source, &destination, &owner(), &SCHEMA).is_err());
        assert_eq!(fs::read(source.join("metrics.json"))?, payload);
        assert_eq!(fs::read(source.join("artifacts/native.tsv"))?, b"replacement native");
        for (name, bytes) in &stable {
            assert_eq!(fs::read(destination.join(name))?, *bytes);
        }
    }
    fs::write(source.join("metrics.json"), vec![b' '; 8 * 1024 * 1024 + 1])?;
    assert!(StagedPublicationPlan::admit(&source, &destination, &owner(), &SCHEMA).is_err());
    for (name, bytes) in stable {
        assert_eq!(fs::read(destination.join(name))?, bytes);
    }
    assert_eq!(fs::read(source.join("artifacts/native.tsv"))?, b"replacement native");
    Ok(())
}

#[test]
fn legacy_replacement_and_owned_retirement_preserve_unknown_files_and_empty_directories(
) -> Result<()> {
    let root = tempfile::tempdir()?;
    let destination = root.path().join("published");
    staged(&destination, b"legacy native")?;
    fs::write(destination.join("notes.txt"), b"user notes")?;
    let first = root.path().join("first");
    staged(&first, b"first native")?;
    fs::create_dir_all(first.join("obsolete"))?;
    fs::write(first.join("obsolete/retired.tsv"), b"obsolete native")?;
    StagedPublicationPlan::admit(&first, &destination, &owner(), &SCHEMA)?.publish()?;
    fs::write(destination.join("artifacts/user.tsv"), b"user artifact")?;
    let second = root.path().join("second");
    staged(&second, b"second native")?;
    StagedPublicationPlan::admit(&second, &destination, &owner(), &SCHEMA)?.publish()?;
    assert_eq!(fs::read(destination.join("artifacts/native.tsv"))?, b"second native");
    assert_eq!(fs::read(destination.join("notes.txt"))?, b"user notes");
    assert_eq!(fs::read(destination.join("artifacts/user.tsv"))?, b"user artifact");
    assert!(!destination.join("obsolete/retired.tsv").exists());
    assert!(destination.join("obsolete").is_dir());
    Ok(())
}

#[test]
fn unowned_collisions_and_reserved_inventory_refuse_without_consuming_inputs() -> Result<()> {
    let root = tempfile::tempdir()?;
    let destination = root.path().join("published");
    fs::create_dir_all(destination.join("artifacts"))?;
    fs::write(destination.join("artifacts/native.tsv"), b"user data")?;
    let source = root.path().join("staged");
    staged(&source, b"replacement native")?;
    assert!(StagedPublicationPlan::admit(&source, &destination, &owner(), &SCHEMA).is_err());
    assert_eq!(fs::read(destination.join("artifacts/native.tsv"))?, b"user data");
    assert_eq!(fs::read(source.join("artifacts/native.tsv"))?, b"replacement native");
    assert!(source.join("metrics.json").is_file());
    fs::write(source.join(SCHEMA.inventory_name), b"invented authority")?;
    let fresh = root.path().join("fresh");
    assert!(StagedPublicationPlan::admit(&source, &fresh, &owner(), &SCHEMA).is_err());
    assert!(!fresh.exists());
    assert!(source.join(SCHEMA.inventory_name).exists());
    Ok(())
}

#[test]
fn overlapping_publication_roots_refuse_before_consuming_evidence() -> Result<()> {
    let root = tempfile::tempdir()?;
    let source = root.path().join("staged");
    staged(&source, b"owned native evidence")?;
    for destination in [source.clone(), source.join("published"), root.path().to_path_buf()] {
        assert!(StagedPublicationPlan::admit(&source, &destination, &owner(), &SCHEMA).is_err());
        assert_eq!(fs::read(source.join("artifacts/native.tsv"))?, b"owned native evidence");
        assert!(source.join("metrics.json").is_file());
        assert!(!source.join(SCHEMA.inventory_name).exists());
    }
    Ok(())
}
