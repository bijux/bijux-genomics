use super::super::location::{path_relative_to_repo, repo_relative_path};
use super::super::paths::{
    bam_schema_path, fastq_schema_path, vcf_schema_path, vcf_stage_schema_dir,
};
use super::contracts::{
    ALL_DOMAIN_SCHEMA_VALIDATION_REPORT_VERSION, DEFAULT_BAM_SCHEMA_VALIDATION_REPORT_PATH,
    DEFAULT_FASTQ_SCHEMA_VALIDATION_REPORT_PATH, DEFAULT_VCF_SCHEMA_VALIDATION_REPORT_PATH,
};
use super::io::write_validation_report;
use super::model::{
    AllDomainSchemaValidationDomainRow, AllDomainSchemaValidationReport, SchemaDomain,
};
use super::shared::{build_bam_schema_validation_report, build_fastq_schema_validation_report};
use super::vcf::build_vcf_schema_validation_report;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
/// Validate all domain schemas.
///
/// # Errors
/// Returns input, canonical schema, filesystem or schema disagreement errors.
pub fn validate_all_domain_schemas(
    repo_root: &Path,
    output_path: &Path,
    schema_root: &Path,
    domains: &[SchemaDomain],
) -> Result<AllDomainSchemaValidationReport> {
    anyhow::ensure!(
        !domains.is_empty()
            && domains.iter().collect::<std::collections::BTreeSet<_>>().len() == domains.len(),
        "schema validation requires nonempty distinct domains"
    );
    let mut rows = Vec::new();
    for domain in domains {
        match domain {
            SchemaDomain::Fastq => {
                let report = build_fastq_schema_validation_report(
                    repo_root,
                    &PathBuf::from(DEFAULT_FASTQ_SCHEMA_VALIDATION_REPORT_PATH),
                    &default_fastq_schema_path(schema_root),
                )?;
                write_validation_report(repo_root, &report.output_path, &report)?;
                rows.push(AllDomainSchemaValidationDomainRow {
                    domain: report.domain.to_string(),
                    output_path: report.output_path.clone(),
                    shared_schema_path: report.shared_schema_path.clone(),
                    stage_dir: None,
                    passes_gate: report.passes_gate,
                    shared_schema_matches: report.shared_schema_matches,
                    stage_count: report.stage_count,
                    required_stage_count: None,
                    exact_stage_schema_file_count: None,
                });
            }
            SchemaDomain::Bam => {
                let report = build_bam_schema_validation_report(
                    repo_root,
                    &PathBuf::from(DEFAULT_BAM_SCHEMA_VALIDATION_REPORT_PATH),
                    &default_bam_schema_path(schema_root),
                )?;
                write_validation_report(repo_root, &report.output_path, &report)?;
                rows.push(AllDomainSchemaValidationDomainRow {
                    domain: report.domain.to_string(),
                    output_path: report.output_path.clone(),
                    shared_schema_path: report.shared_schema_path.clone(),
                    stage_dir: None,
                    passes_gate: report.passes_gate,
                    shared_schema_matches: report.shared_schema_matches,
                    stage_count: report.stage_count,
                    required_stage_count: None,
                    exact_stage_schema_file_count: None,
                });
            }
            SchemaDomain::Vcf => {
                let report = build_vcf_schema_validation_report(
                    repo_root,
                    &PathBuf::from(DEFAULT_VCF_SCHEMA_VALIDATION_REPORT_PATH),
                    &default_vcf_schema_path(schema_root),
                    &default_vcf_stage_dir(schema_root),
                )?;
                write_validation_report(repo_root, &report.output_path, &report)?;
                rows.push(AllDomainSchemaValidationDomainRow {
                    domain: report.domain.to_string(),
                    output_path: report.output_path.clone(),
                    shared_schema_path: report.shared_schema_path.clone(),
                    stage_dir: Some(report.stage_dir.clone()),
                    passes_gate: report.passes_gate,
                    shared_schema_matches: report.shared_schema_matches,
                    stage_count: report.stage_count,
                    required_stage_count: Some(report.required_stage_count),
                    exact_stage_schema_file_count: Some(report.exact_stage_schema_file_count),
                });
            }
        }
    }
    rows.sort_by(|left, right| left.domain.cmp(&right.domain));

    let absolute_output_path = repo_relative_path(repo_root, output_path);
    let schema_root = repo_relative_path(repo_root, schema_root);
    let passed_domain_count = rows.iter().filter(|row| row.passes_gate).count();
    let failed_domain_count = rows.len().saturating_sub(passed_domain_count);
    let report = AllDomainSchemaValidationReport {
        schema_version: ALL_DOMAIN_SCHEMA_VALIDATION_REPORT_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        schema_root: path_relative_to_repo(repo_root, &schema_root),
        domain_count: rows.len(),
        passed_domain_count,
        failed_domain_count,
        ok: failed_domain_count == 0,
        domains: rows,
    };
    write_validation_report(repo_root, &report.output_path, &report)?;
    if !report.ok {
        bail!("all-domain schema validation failed under `{}`", report.schema_root);
    }
    Ok(report)
}
fn default_fastq_schema_path(schema_root: &Path) -> PathBuf {
    fastq_schema_path(schema_root)
}
fn default_bam_schema_path(schema_root: &Path) -> PathBuf {
    bam_schema_path(schema_root)
}
fn default_vcf_schema_path(schema_root: &Path) -> PathBuf {
    vcf_schema_path(schema_root)
}
fn default_vcf_stage_dir(schema_root: &Path) -> PathBuf {
    vcf_stage_schema_dir(schema_root)
}
