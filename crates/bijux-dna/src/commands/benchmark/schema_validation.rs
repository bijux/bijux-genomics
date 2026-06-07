use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::schema_paths::{
    DEFAULT_BAM_NORMALIZED_METRICS_SCHEMA_PATH, DEFAULT_BENCHMARK_SCHEMA_ROOT,
    DEFAULT_FASTQ_NORMALIZED_METRICS_SCHEMA_PATH, DEFAULT_VCF_NORMALIZED_METRICS_SCHEMA_PATH,
    DEFAULT_VCF_NORMALIZED_METRICS_STAGE_DIR,
};
use crate::commands::cli::parse::{self, BenchSchemaDomainArg};
use crate::commands::cli::render;

pub(crate) const DEFAULT_FASTQ_SCHEMA_VALIDATION_REPORT_PATH: &str =
    "target/bench-readiness/fastq-schema-validation.json";
pub(crate) const DEFAULT_BAM_SCHEMA_VALIDATION_REPORT_PATH: &str =
    "target/bench-readiness/bam-schema-validation.json";
pub(crate) const DEFAULT_VCF_SCHEMA_VALIDATION_REPORT_PATH: &str =
    "target/bench-readiness/vcf-schema-validation.json";
pub(crate) const DEFAULT_ALL_DOMAIN_SCHEMA_VALIDATION_REPORT_PATH: &str =
    "target/bench-readiness/all-domain-schema-validation.json";

const FASTQ_SCHEMA_VALIDATION_REPORT_VERSION: &str =
    "bijux.bench.readiness.fastq_schema_validation.v1";
const BAM_SCHEMA_VALIDATION_REPORT_VERSION: &str = "bijux.bench.readiness.bam_schema_validation.v1";
const VCF_SCHEMA_VALIDATION_REPORT_VERSION: &str = "bijux.bench.readiness.vcf_schema_validation.v1";
const ALL_DOMAIN_SCHEMA_VALIDATION_REPORT_VERSION: &str =
    "bijux.bench.readiness.all_domain_schema_validation.v1";

const REQUIRED_VCF_SCHEMA_STAGE_IDS: &[&str] = &[
    "vcf.call",
    "vcf.call_diploid",
    "vcf.call_pseudohaploid",
    "vcf.call_gl",
    "vcf.damage_filter",
    "vcf.filter",
    "vcf.stats",
    "vcf.qc",
    "vcf.prepare_reference_panel",
    "vcf.phasing",
    "vcf.imputation",
    "vcf.pca",
    "vcf.admixture",
    "vcf.population_structure",
    "vcf.roh",
    "vcf.ibd",
    "vcf.demography",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct SharedSchemaValidationRow {
    pub(crate) stage_id: String,
    pub(crate) extension_id: String,
    pub(crate) required_key_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqSchemaValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) output_path: String,
    pub(crate) shared_schema_path: String,
    pub(crate) passes_gate: bool,
    pub(crate) shared_schema_matches: bool,
    pub(crate) stage_count: usize,
    pub(crate) rows: Vec<SharedSchemaValidationRow>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamSchemaValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) output_path: String,
    pub(crate) shared_schema_path: String,
    pub(crate) passes_gate: bool,
    pub(crate) shared_schema_matches: bool,
    pub(crate) stage_count: usize,
    pub(crate) rows: Vec<SharedSchemaValidationRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfSchemaValidationRow {
    pub(crate) stage_id: String,
    pub(crate) schema_version: String,
    pub(crate) schema_file: String,
    pub(crate) file_present: bool,
    pub(crate) exact_match: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfSchemaValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) output_path: String,
    pub(crate) shared_schema_path: String,
    pub(crate) stage_dir: String,
    pub(crate) passes_gate: bool,
    pub(crate) shared_schema_matches: bool,
    pub(crate) stage_count: usize,
    pub(crate) required_stage_count: usize,
    pub(crate) exact_stage_schema_file_count: usize,
    pub(crate) missing_stage_schema_files: Vec<String>,
    pub(crate) unexpected_stage_schema_files: Vec<String>,
    pub(crate) rows: Vec<VcfSchemaValidationRow>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainSchemaValidationDomainRow {
    pub(crate) domain: String,
    pub(crate) output_path: String,
    pub(crate) shared_schema_path: String,
    pub(crate) stage_dir: Option<String>,
    pub(crate) passes_gate: bool,
    pub(crate) shared_schema_matches: bool,
    pub(crate) stage_count: usize,
    pub(crate) required_stage_count: Option<usize>,
    pub(crate) exact_stage_schema_file_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainSchemaValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) schema_root: String,
    pub(crate) domain_count: usize,
    pub(crate) passed_domain_count: usize,
    pub(crate) failed_domain_count: usize,
    pub(crate) ok: bool,
    pub(crate) domains: Vec<AllDomainSchemaValidationDomainRow>,
}

pub(crate) fn run_validate_schemas(
    repo_root: &Path,
    args: &parse::BenchValidateSchemasArgs,
) -> Result<()> {
    let domains = normalize_domains(&args.domain);
    if domains.is_empty() {
        bail!("at least one schema domain is required");
    }
    let schema_root =
        args.schema_root.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BENCHMARK_SCHEMA_ROOT));

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

pub(crate) fn validate_fastq_schemas(
    repo_root: &Path,
    output_path: PathBuf,
    shared_schema_path: PathBuf,
) -> Result<FastqSchemaValidationReport> {
    let report = build_fastq_schema_validation_report(repo_root, output_path, shared_schema_path)?;
    write_validation_report(repo_root, &report.output_path, &report)?;
    if !report.passes_gate {
        bail!("FASTQ schema validation failed for `{}`", report.shared_schema_path);
    }
    Ok(report)
}

pub(crate) fn validate_bam_schemas(
    repo_root: &Path,
    output_path: PathBuf,
    shared_schema_path: PathBuf,
) -> Result<BamSchemaValidationReport> {
    let report = build_bam_schema_validation_report(repo_root, output_path, shared_schema_path)?;
    write_validation_report(repo_root, &report.output_path, &report)?;
    if !report.passes_gate {
        bail!("BAM schema validation failed for `{}`", report.shared_schema_path);
    }
    Ok(report)
}

pub(crate) fn validate_vcf_schemas(
    repo_root: &Path,
    output_path: PathBuf,
    shared_schema_path: PathBuf,
    stage_dir: PathBuf,
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

fn validate_all_domain_schemas(
    repo_root: &Path,
    output_path: PathBuf,
    schema_root: &Path,
    domains: &[BenchSchemaDomainArg],
) -> Result<AllDomainSchemaValidationReport> {
    let mut rows = Vec::new();
    for domain in domains {
        match domain {
            BenchSchemaDomainArg::Fastq => {
                let report = build_fastq_schema_validation_report(
                    repo_root,
                    PathBuf::from(DEFAULT_FASTQ_SCHEMA_VALIDATION_REPORT_PATH),
                    default_fastq_schema_path(schema_root),
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
            BenchSchemaDomainArg::Bam => {
                let report = build_bam_schema_validation_report(
                    repo_root,
                    PathBuf::from(DEFAULT_BAM_SCHEMA_VALIDATION_REPORT_PATH),
                    default_bam_schema_path(schema_root),
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
            BenchSchemaDomainArg::Vcf => {
                let report = build_vcf_schema_validation_report(
                    repo_root,
                    PathBuf::from(DEFAULT_VCF_SCHEMA_VALIDATION_REPORT_PATH),
                    default_vcf_schema_path(schema_root),
                    default_vcf_stage_dir(schema_root),
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

    let absolute_output_path = repo_relative_path(repo_root, &output_path);
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

fn build_fastq_schema_validation_report(
    repo_root: &Path,
    output_path: PathBuf,
    shared_schema_path: PathBuf,
) -> Result<FastqSchemaValidationReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let shared_schema_path = repo_relative_path(repo_root, &shared_schema_path);
    let canonical_shared = bijux_dna_api::v1::api::bench::render_fastq_normalized_metrics_schema();
    let disk_shared = read_json_value(&shared_schema_path)?;
    let rows = collect_shared_schema_rows(&canonical_shared, "FASTQ")?;

    Ok(FastqSchemaValidationReport {
        schema_version: FASTQ_SCHEMA_VALIDATION_REPORT_VERSION,
        domain: "fastq",
        output_path: path_relative_to_repo(repo_root, &output_path),
        shared_schema_path: path_relative_to_repo(repo_root, &shared_schema_path),
        passes_gate: disk_shared == canonical_shared,
        shared_schema_matches: disk_shared == canonical_shared,
        stage_count: rows.len(),
        rows,
    })
}

fn build_bam_schema_validation_report(
    repo_root: &Path,
    output_path: PathBuf,
    shared_schema_path: PathBuf,
) -> Result<BamSchemaValidationReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let shared_schema_path = repo_relative_path(repo_root, &shared_schema_path);
    let canonical_shared = bijux_dna_api::v1::api::bench::render_bam_normalized_metrics_schema();
    let disk_shared = read_json_value(&shared_schema_path)?;
    let rows = collect_shared_schema_rows(&canonical_shared, "BAM")?;

    Ok(BamSchemaValidationReport {
        schema_version: BAM_SCHEMA_VALIDATION_REPORT_VERSION,
        domain: "bam",
        output_path: path_relative_to_repo(repo_root, &output_path),
        shared_schema_path: path_relative_to_repo(repo_root, &shared_schema_path),
        passes_gate: disk_shared == canonical_shared,
        shared_schema_matches: disk_shared == canonical_shared,
        stage_count: rows.len(),
        rows,
    })
}

fn build_vcf_schema_validation_report(
    repo_root: &Path,
    output_path: PathBuf,
    shared_schema_path: PathBuf,
    stage_dir: PathBuf,
) -> Result<VcfSchemaValidationReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let shared_schema_path = repo_relative_path(repo_root, &shared_schema_path);
    let stage_dir = repo_relative_path(repo_root, &stage_dir);

    let canonical_shared = bijux_dna_api::v1::api::bench::render_vcf_normalized_metrics_schema()?;
    let descriptors =
        bijux_dna_api::v1::api::bench::vcf_normalized_metrics_stage_schema_descriptors()?;

    let disk_shared = read_json_value(&shared_schema_path)?;
    let shared_schema_matches = disk_shared == canonical_shared;

    let required_stage_ids = REQUIRED_VCF_SCHEMA_STAGE_IDS
        .iter()
        .map(|stage_id| (*stage_id).to_string())
        .collect::<BTreeSet<_>>();
    let declared_stage_ids =
        descriptors.iter().map(|descriptor| descriptor.stage_id.clone()).collect::<BTreeSet<_>>();
    let missing_required_stage_ids =
        required_stage_ids.difference(&declared_stage_ids).cloned().collect::<Vec<_>>();
    if !missing_required_stage_ids.is_empty() {
        bail!(
            "VCF normalized metrics schema contract is missing required stages: {}",
            missing_required_stage_ids.join(", ")
        );
    }

    let actual_stage_files = fs::read_dir(&stage_dir)
        .with_context(|| format!("read {}", stage_dir.display()))?
        .map(|entry| {
            let entry = entry?;
            Ok((entry.file_name().to_string_lossy().to_string(), entry.file_type()?.is_file()))
        })
        .collect::<Result<Vec<_>>>()?;
    let actual_stage_files = actual_stage_files
        .into_iter()
        .filter_map(|(name, is_file)| is_file.then_some(name))
        .collect::<BTreeSet<_>>();
    let expected_stage_files =
        descriptors.iter().map(|descriptor| descriptor.file_name.clone()).collect::<BTreeSet<_>>();

    let missing_stage_schema_files =
        expected_stage_files.difference(&actual_stage_files).cloned().collect::<Vec<_>>();
    let unexpected_stage_schema_files =
        actual_stage_files.difference(&expected_stage_files).cloned().collect::<Vec<_>>();

    let mut exact_stage_schema_file_count = 0usize;
    let mut rows = Vec::with_capacity(descriptors.len());
    for descriptor in &descriptors {
        let schema_path = stage_dir.join(&descriptor.file_name);
        let canonical_stage =
            bijux_dna_api::v1::api::bench::render_vcf_normalized_metrics_stage_schema(
                &descriptor.stage_id,
            )?;
        let (file_present, exact_match) = if schema_path.exists() {
            let disk_stage = read_json_value(&schema_path)?;
            let exact_match = disk_stage == canonical_stage;
            (true, exact_match)
        } else {
            (false, false)
        };
        if exact_match {
            exact_stage_schema_file_count += 1;
        }
        rows.push(VcfSchemaValidationRow {
            stage_id: descriptor.stage_id.clone(),
            schema_version: descriptor.schema_version.clone(),
            schema_file: path_relative_to_repo(repo_root, &schema_path),
            file_present,
            exact_match,
        });
    }
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));

    let passes_gate = shared_schema_matches
        && missing_stage_schema_files.is_empty()
        && unexpected_stage_schema_files.is_empty()
        && exact_stage_schema_file_count == descriptors.len();

    Ok(VcfSchemaValidationReport {
        schema_version: VCF_SCHEMA_VALIDATION_REPORT_VERSION,
        domain: "vcf",
        output_path: path_relative_to_repo(repo_root, &output_path),
        shared_schema_path: path_relative_to_repo(repo_root, &shared_schema_path),
        stage_dir: path_relative_to_repo(repo_root, &stage_dir),
        passes_gate,
        shared_schema_matches,
        stage_count: descriptors.len(),
        required_stage_count: REQUIRED_VCF_SCHEMA_STAGE_IDS.len(),
        exact_stage_schema_file_count,
        missing_stage_schema_files,
        unexpected_stage_schema_files,
        rows,
    })
}

fn collect_shared_schema_rows(
    schema: &serde_json::Value,
    domain_name: &str,
) -> Result<Vec<SharedSchemaValidationRow>> {
    let stage_defs = schema
        .get("$defs")
        .and_then(|value| value.get("stages"))
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            anyhow!("{domain_name} normalized metrics schema is missing object `$defs.stages`")
        })?;

    let mut rows = stage_defs
        .iter()
        .map(|(stage_id, value)| {
            let stage_contract = value
                .get("allOf")
                .and_then(serde_json::Value::as_array)
                .and_then(|items| items.get(1))
                .ok_or_else(|| anyhow!("{domain_name} normalized metrics stage `{stage_id}` is missing stage extension"))?;
            let extension_id = stage_contract
                .get("x-bijux-extension-id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow!("{domain_name} normalized metrics stage `{stage_id}` is missing string `x-bijux-extension-id`"))?;
            let required_key_count = stage_contract
                .get("required")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| anyhow!("{domain_name} normalized metrics stage `{stage_id}` is missing `required` keys"))?
                .len();
            Ok(SharedSchemaValidationRow {
                stage_id: stage_id.clone(),
                extension_id: extension_id.to_string(),
                required_key_count,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    Ok(rows)
}

fn normalize_domains(domains: &[BenchSchemaDomainArg]) -> Vec<BenchSchemaDomainArg> {
    let mut unique = BTreeSet::new();
    for domain in domains {
        unique.insert(*domain);
    }
    unique.into_iter().collect()
}

fn read_json_value(path: &Path) -> Result<serde_json::Value> {
    serde_json::from_slice(&fs::read(path).with_context(|| format!("read {}", path.display()))?)
        .with_context(|| format!("parse {}", path.display()))
}

fn write_validation_report<T: Serialize>(
    repo_root: &Path,
    output_path: &str,
    report: &T,
) -> Result<()> {
    let absolute_output_path = repo_root.join(output_path);
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&absolute_output_path, report)
        .with_context(|| format!("write {}", absolute_output_path.display()))?;
    Ok(())
}

fn default_fastq_schema_path(schema_root: &Path) -> PathBuf {
    if schema_root == Path::new(DEFAULT_BENCHMARK_SCHEMA_ROOT) {
        PathBuf::from(DEFAULT_FASTQ_NORMALIZED_METRICS_SCHEMA_PATH)
    } else {
        schema_root.join("fastq-normalized-metrics.v1.json")
    }
}

fn default_bam_schema_path(schema_root: &Path) -> PathBuf {
    if schema_root == Path::new(DEFAULT_BENCHMARK_SCHEMA_ROOT) {
        PathBuf::from(DEFAULT_BAM_NORMALIZED_METRICS_SCHEMA_PATH)
    } else {
        schema_root.join("bam-normalized-metrics.v1.json")
    }
}

fn default_vcf_schema_path(schema_root: &Path) -> PathBuf {
    if schema_root == Path::new(DEFAULT_BENCHMARK_SCHEMA_ROOT) {
        PathBuf::from(DEFAULT_VCF_NORMALIZED_METRICS_SCHEMA_PATH)
    } else {
        schema_root.join("vcf-normalized-metrics.v1.json")
    }
}

fn default_vcf_stage_dir(schema_root: &Path) -> PathBuf {
    if schema_root == Path::new(DEFAULT_BENCHMARK_SCHEMA_ROOT) {
        PathBuf::from(DEFAULT_VCF_NORMALIZED_METRICS_STAGE_DIR)
    } else {
        schema_root.join("vcf-normalized-metrics")
    }
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
