use super::{
    io::write_validation_report,
    model::{BamSchemaValidationReport, FastqSchemaValidationReport, VcfSchemaValidationReport},
    shared::{build_bam_schema_validation_report, build_fastq_schema_validation_report},
    vcf::build_vcf_schema_validation_report,
};
use anyhow::{bail, Result};
use std::path::Path;
/// Validate fastq schemas.
///
/// # Errors
/// Returns input, canonical schema, filesystem or schema disagreement errors.
pub fn validate_fastq_schemas(
    repo_root: &Path,
    output_path: &Path,
    shared_schema_path: &Path,
) -> Result<FastqSchemaValidationReport> {
    let report = build_fastq_schema_validation_report(repo_root, output_path, shared_schema_path)?;
    write_validation_report(repo_root, &report.output_path, &report)?;
    if !report.passes_gate {
        bail!("FASTQ schema validation failed for `{}`", report.shared_schema_path);
    }
    Ok(report)
}
/// Validate bam schemas.
///
/// # Errors
/// Returns input, canonical schema, filesystem or schema disagreement errors.
pub fn validate_bam_schemas(
    repo_root: &Path,
    output_path: &Path,
    shared_schema_path: &Path,
) -> Result<BamSchemaValidationReport> {
    let report = build_bam_schema_validation_report(repo_root, output_path, shared_schema_path)?;
    write_validation_report(repo_root, &report.output_path, &report)?;
    if !report.passes_gate {
        bail!("BAM schema validation failed for `{}`", report.shared_schema_path);
    }
    Ok(report)
}
/// Validate vcf schemas.
///
/// # Errors
/// Returns input, canonical schema, filesystem or schema disagreement errors.
pub fn validate_vcf_schemas(
    repo_root: &Path,
    output_path: &Path,
    shared_schema_path: &Path,
    stage_dir: &Path,
) -> Result<VcfSchemaValidationReport> {
    let report =
        build_vcf_schema_validation_report(repo_root, output_path, shared_schema_path, stage_dir)?;
    write_validation_report(repo_root, &report.output_path, &report)?;
    if !report.passes_gate {
        bail!(
            "VCF schema validation failed for `{}` and `{}`",
            report.shared_schema_path,
            report.stage_dir
        );
    }
    Ok(report)
}
