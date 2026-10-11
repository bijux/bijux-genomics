use super::support::readiness::RejectedSource;
use anyhow::{Context, Result};
use bijux_dna_bench_bam_evidence::readiness;
use std::{fs, path::Path};

#[test]
fn contamination_sex_haplogroups_rejects_source_without_native_effects_or_report_overwrite(
) -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = RejectedSource::default();
    fs::write(root.path().join("retained.json"), "previous evidence")?;
    let error =
        readiness::contamination_sex_haplogroups::render_bam_contamination_sex_haplogroups_ready(
            &evidence.context(root.path()),
            Path::new("retained.json"),
        )
        .err()
        .context("source rejection must propagate")?;
    assert_eq!(error.to_string(), "captured serving source rejection");
    assert_eq!(*evidence.calls.borrow(), ["serving"]);
    assert_eq!(fs::read_to_string(root.path().join("retained.json"))?, "previous evidence");
    Ok(())
}

#[test]
fn recalibration_genotyping_rejects_source_without_native_effects_or_report_overwrite() -> Result<()>
{
    let root = tempfile::tempdir()?;
    let evidence = RejectedSource::default();
    fs::write(root.path().join("retained.json"), "previous evidence")?;
    let error = readiness::recalibration_genotyping::render_bam_recalibration_genotyping_ready(
        &evidence.context(root.path()),
        Path::new("retained.json"),
    )
    .err()
    .context("source rejection must propagate")?;
    assert_eq!(error.to_string(), "captured serving source rejection");
    assert_eq!(*evidence.calls.borrow(), ["serving"]);
    assert_eq!(fs::read_to_string(root.path().join("retained.json"))?, "previous evidence");
    Ok(())
}

#[test]
fn damage_authenticity_rejects_source_without_native_effects_or_report_overwrite() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = RejectedSource::default();
    fs::write(root.path().join("retained.json"), "previous evidence")?;
    let error = readiness::damage_authenticity::render_bam_damage_authenticity_ready(
        &evidence.context(root.path()),
        Path::new("retained.json"),
    )
    .err()
    .context("source rejection must propagate")?;
    assert_eq!(error.to_string(), "captured serving source rejection");
    assert_eq!(*evidence.calls.borrow(), ["serving"]);
    assert_eq!(fs::read_to_string(root.path().join("retained.json"))?, "previous evidence");
    Ok(())
}

#[test]
fn insert_size_gc_bias_rejects_source_without_native_effects_or_report_overwrite() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = RejectedSource::default();
    fs::write(root.path().join("retained.json"), "previous evidence")?;
    let error = readiness::insert_size_gc_bias::render_bam_insert_size_gc_bias_ready(
        &evidence.context(root.path()),
        Path::new("retained.json"),
    )
    .err()
    .context("source rejection must propagate")?;
    assert_eq!(error.to_string(), "captured serving source rejection");
    assert_eq!(*evidence.calls.borrow(), ["serving"]);
    assert_eq!(fs::read_to_string(root.path().join("retained.json"))?, "previous evidence");
    Ok(())
}

#[test]
fn overlap_endogenous_rejects_source_without_native_effects_or_report_overwrite() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = RejectedSource::default();
    fs::write(root.path().join("retained.json"), "previous evidence")?;
    let error = readiness::overlap_endogenous::render_bam_overlap_endogenous_ready(
        &evidence.context(root.path()),
        Path::new("retained.json"),
    )
    .err()
    .context("source rejection must propagate")?;
    assert_eq!(error.to_string(), "captured serving source rejection");
    assert_eq!(*evidence.calls.borrow(), ["serving"]);
    assert_eq!(fs::read_to_string(root.path().join("retained.json"))?, "previous evidence");
    Ok(())
}

#[test]
fn kinship_rejects_source_without_native_effects_or_report_overwrite() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = RejectedSource::default();
    fs::write(root.path().join("retained.json"), "previous evidence")?;
    let error = readiness::kinship::render_bam_kinship_ready(
        &evidence.context(root.path()),
        Path::new("retained.json"),
    )
    .err()
    .context("source rejection must propagate")?;
    assert_eq!(error.to_string(), "captured serving source rejection");
    assert_eq!(*evidence.calls.borrow(), ["serving"]);
    assert_eq!(fs::read_to_string(root.path().join("retained.json"))?, "previous evidence");
    Ok(())
}
