use super::{validate_required_files, InputContext};
use anyhow::Result;
use std::fs;
#[test]
fn required_file_admission_rejects_directories_and_malformed_identity() -> Result<()> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("valid.vcf"), b"fixture")?;
    fs::create_dir(root.path().join("directory.vcf"))?;
    validate_required_files(root.path(), "vcf.roh", InputContext::Audit, [("vcf", "valid.vcf")])?;
    for (stage, inputs) in [
        ("vcf.roh", vec![("vcf", "directory.vcf")]),
        ("vcf.roh", vec![("vcf", "absent.vcf")]),
        ("vcf.roh", vec![("vcf", "valid.vcf"), ("vcf", "valid.vcf")]),
        ("vcf.roh", vec![(" padded ", "valid.vcf")]),
        (" padded ", vec![("vcf", "valid.vcf")]),
        ("vcf.roh", vec![]),
    ] {
        assert!(validate_required_files(root.path(), stage, InputContext::Audit, inputs).is_err());
    }
    Ok(())
}
#[test]
fn family_and_audit_diagnostics_report_the_actual_missing_artifact() -> Result<()> {
    let root = tempfile::tempdir()?;
    for family in [
        "bcftools",
        "angsd",
        "plink-family",
        "phasing-family",
        "eigensoft",
        "imputation-family",
        "descent-family",
    ] {
        let error = validate_required_files(
            root.path(),
            "vcf.call",
            InputContext::Adapter { family, descent: family == "descent-family" },
            [("reference_fasta", "missing.fa")],
        )
        .err()
        .ok_or_else(|| anyhow::anyhow!("missing input accepted"))?;
        assert!(error.to_string().contains(family));
        assert!(error.to_string().contains("required input `reference_fasta`"));
        assert!(error.to_string().contains("missing.fa"));
    }
    Ok(())
}
