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

fn rejects_acquired_damage_binding_before_native(
    mutate: fn(&mut super::support::readiness::preflight::AcquiredDeclarations),
    expected_error: &str,
) -> Result<()> {
    use super::support::readiness::preflight::AcquiredDeclarations;
    let root = tempfile::tempdir()?;
    let mut declarations = AcquiredDeclarations::damage_authenticity(root.path())?;
    mutate(&mut declarations);
    let output = root.path().join("retained.json");
    let previous = b"previous scientific evidence\n";
    fs::write(&output, previous)?;
    let error = readiness::damage_authenticity::render_bam_damage_authenticity_ready(
        &declarations.context(root.path()),
        Path::new("retained.json"),
    )
    .err()
    .context("invalid acquired binding must reject")?;
    assert_eq!(
        *declarations.calls.borrow(),
        ["serving", "commands", "outputs", "parsers", "expected", "report-map", "schema"]
    );
    assert_eq!(fs::read(&output)?, previous);
    assert!(
        declarations.native_calls.borrow().is_empty(),
        "invalid retained declarations reached native operations: {:?}",
        *declarations.native_calls.borrow()
    );
    assert!(error.to_string().contains(expected_error), "unexpected admission error: {error}");
    Ok(())
}

#[test]
fn damage_authenticity_missing_acquired_binding_refuses_before_native_effects() -> Result<()> {
    use super::support::readiness::preflight::AcquiredDeclarations;
    rejects_acquired_damage_binding_before_native(
        AcquiredDeclarations::remove_retained_binding,
        "BAM damage/authenticity active rows must cover",
    )
}

#[test]
fn damage_authenticity_duplicate_acquired_binding_refuses_before_native_effects() -> Result<()> {
    use super::support::readiness::preflight::AcquiredDeclarations;
    rejects_acquired_damage_binding_before_native(
        AcquiredDeclarations::duplicate_retained_binding,
        "duplicate BAM damage/authenticity binding `bam.authenticity` / `authenticct`",
    )
}

#[test]
fn damage_authenticity_substituted_stage_refuses_before_native_effects() -> Result<()> {
    use super::support::readiness::preflight::AcquiredDeclarations;
    rejects_acquired_damage_binding_before_native(
        AcquiredDeclarations::substitute_foreign_stage,
        "BAM damage/authenticity active rows must cover",
    )
}

#[test]
fn admitted_damage_declarations_preserve_first_native_operation_and_error() -> Result<()> {
    use super::support::readiness::preflight::AcquiredDeclarations;
    use bijux_dna_bench_bam_evidence::readiness::ports::LocalProofKind;
    let root = tempfile::tempdir()?;
    let declarations = AcquiredDeclarations::damage_authenticity(root.path())?;
    let output = root.path().join("retained.json");
    let previous = b"previous scientific evidence\n";
    fs::write(&output, previous)?;
    let error = readiness::damage_authenticity::render_bam_damage_authenticity_ready(
        &declarations.context(root.path()),
        Path::new("retained.json"),
    )
    .err()
    .context("captured native error must propagate")?;
    assert_eq!(error.to_string(), "captured native boundary reached");
    assert_eq!(*declarations.native_calls.borrow(), [LocalProofKind::DamageSmoke]);
    assert_eq!(
        *declarations.calls.borrow(),
        ["serving", "commands", "outputs", "parsers", "expected", "report-map", "schema"]
    );
    assert_eq!(fs::read(&output)?, previous);
    Ok(())
}
