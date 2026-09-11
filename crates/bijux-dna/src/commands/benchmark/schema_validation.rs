use std::collections::BTreeSet;

use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use crate::commands::benchmark::schema_paths::{
    bam_schema_path, benchmark_schema_root_path, fastq_schema_path, vcf_schema_path,
    vcf_stage_schema_dir,
};
use crate::commands::cli::parse::{self, BenchSchemaDomainArg};
use crate::commands::cli::render;

pub(crate) use bijux_dna_bench_artifacts::schemas::validation::{
    AllDomainSchemaValidationReport, BamSchemaValidationReport, FastqSchemaValidationReport,
    VcfSchemaValidationReport, DEFAULT_ALL_DOMAIN_SCHEMA_VALIDATION_REPORT_PATH,
    DEFAULT_BAM_SCHEMA_VALIDATION_REPORT_PATH, DEFAULT_FASTQ_SCHEMA_VALIDATION_REPORT_PATH,
    DEFAULT_VCF_SCHEMA_VALIDATION_REPORT_PATH,
};
pub(crate) fn run_validate_schemas(
    repo_root: &Path,
    args: &parse::BenchValidateSchemasArgs,
) -> Result<()> {
    let domains = normalize_domains(&args.domain);
    if domains.is_empty() {
        bail!("at least one schema domain is required");
    }
    let schema_root = benchmark_schema_root_path(repo_root, args.schema_root.as_deref());

    if domains.len() == 1 {
        run_single_domain_validation(repo_root, &domains[0], &schema_root, args)
    } else {
        if args.shared_schema.is_some() || args.stage_dir.is_some() {
            bail!("--shared-schema and --stage-dir require single-domain schema validation");
        }
        let report = validate_all_domain_schemas(
            repo_root,
            args.output
                .clone()
                .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_SCHEMA_VALIDATION_REPORT_PATH)),
            &schema_root,
            &domains,
        )?;
        if args.json {
            render::json::print_pretty(&report)?;
        } else {
            println!("{}", report.output_path);
        }
        Ok(())
    }
}
fn run_single_domain_validation(
    repo_root: &Path,
    domain: &BenchSchemaDomainArg,
    schema_root: &Path,
    args: &parse::BenchValidateSchemasArgs,
) -> Result<()> {
    match domain {
        BenchSchemaDomainArg::Fastq => {
            if args.stage_dir.is_some() {
                bail!("--stage-dir is not valid for FASTQ schema validation");
            }
            let report = validate_fastq_schemas(
                repo_root,
                args.output
                    .clone()
                    .unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_SCHEMA_VALIDATION_REPORT_PATH)),
                args.shared_schema
                    .clone()
                    .unwrap_or_else(|| default_fastq_schema_path(schema_root)),
            )?;
            if args.json {
                render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.output_path);
            }
        }
        BenchSchemaDomainArg::Bam => {
            if args.stage_dir.is_some() {
                bail!("--stage-dir is not valid for BAM schema validation");
            }
            let report = validate_bam_schemas(
                repo_root,
                args.output
                    .clone()
                    .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_SCHEMA_VALIDATION_REPORT_PATH)),
                args.shared_schema.clone().unwrap_or_else(|| default_bam_schema_path(schema_root)),
            )?;
            if args.json {
                render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.output_path);
            }
        }
        BenchSchemaDomainArg::Vcf => {
            let report = validate_vcf_schemas(
                repo_root,
                args.output
                    .clone()
                    .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_SCHEMA_VALIDATION_REPORT_PATH)),
                args.shared_schema.clone().unwrap_or_else(|| default_vcf_schema_path(schema_root)),
                args.stage_dir.clone().unwrap_or_else(|| default_vcf_stage_dir(schema_root)),
            )?;
            if args.json {
                render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.output_path);
            }
        }
    }
    Ok(())
}
fn normalize_domains(domains: &[BenchSchemaDomainArg]) -> Vec<BenchSchemaDomainArg> {
    let mut unique = BTreeSet::new();
    for domain in domains {
        unique.insert(*domain);
    }
    unique.into_iter().collect()
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
pub(crate) fn validate_fastq_schemas(
    repo_root: &Path,
    output_path: PathBuf,
    shared_schema_path: PathBuf,
) -> Result<FastqSchemaValidationReport> {
    bijux_dna_bench_artifacts::schemas::validation::validate_fastq_schemas(
        repo_root,
        &output_path,
        &shared_schema_path,
    )
}
pub(crate) fn validate_bam_schemas(
    repo_root: &Path,
    output_path: PathBuf,
    shared_schema_path: PathBuf,
) -> Result<BamSchemaValidationReport> {
    bijux_dna_bench_artifacts::schemas::validation::validate_bam_schemas(
        repo_root,
        &output_path,
        &shared_schema_path,
    )
}
pub(crate) fn validate_vcf_schemas(
    repo_root: &Path,
    output_path: PathBuf,
    shared_schema_path: PathBuf,
    stage_dir: PathBuf,
) -> Result<VcfSchemaValidationReport> {
    bijux_dna_bench_artifacts::schemas::validation::validate_vcf_schemas(
        repo_root,
        &output_path,
        &shared_schema_path,
        &stage_dir,
    )
}
pub(crate) fn validate_all_domain_schemas(
    repo_root: &Path,
    output_path: PathBuf,
    schema_root: &Path,
    domains: &[BenchSchemaDomainArg],
) -> Result<AllDomainSchemaValidationReport> {
    use bijux_dna_bench_artifacts::schemas::validation::SchemaDomain;
    let domains = domains
        .iter()
        .map(|domain| match domain {
            BenchSchemaDomainArg::Fastq => SchemaDomain::Fastq,
            BenchSchemaDomainArg::Bam => SchemaDomain::Bam,
            BenchSchemaDomainArg::Vcf => SchemaDomain::Vcf,
        })
        .collect::<Vec<_>>();
    bijux_dna_bench_artifacts::schemas::validation::validate_all_domain_schemas(
        repo_root,
        &output_path,
        schema_root,
        &domains,
    )
}
