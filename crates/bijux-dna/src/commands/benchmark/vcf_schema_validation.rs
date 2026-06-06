use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::ValueEnum;
use serde::Serialize;

use crate::commands::benchmark::readiness::vcf_normalized_metrics_schema::{
    DEFAULT_VCF_NORMALIZED_METRICS_SCHEMA_PATH, DEFAULT_VCF_NORMALIZED_METRICS_STAGE_DIR,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_SCHEMA_VALIDATION_REPORT_PATH: &str =
    "target/bench-readiness/vcf-schema-validation.json";
const VCF_SCHEMA_VALIDATION_REPORT_VERSION: &str = "bijux.bench.readiness.vcf_schema_validation.v1";
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum BenchSchemaDomainArg {
    Vcf,
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

pub(crate) fn run_validate_vcf_schemas(
    repo_root: &Path,
    args: &parse::BenchValidateSchemasArgs,
) -> Result<()> {
    let report = validate_vcf_schemas(
        repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_SCHEMA_VALIDATION_REPORT_PATH)),
        args.shared_schema
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_NORMALIZED_METRICS_SCHEMA_PATH)),
        args.stage_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_NORMALIZED_METRICS_STAGE_DIR)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn validate_vcf_schemas(
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

    let disk_shared: serde_json::Value = serde_json::from_slice(
        &fs::read(&shared_schema_path)
            .with_context(|| format!("read {}", shared_schema_path.display()))?,
    )
    .with_context(|| format!("parse {}", shared_schema_path.display()))?;
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
            let disk_stage: serde_json::Value = serde_json::from_slice(
                &fs::read(&schema_path)
                    .with_context(|| format!("read {}", schema_path.display()))?,
            )
            .with_context(|| format!("parse {}", schema_path.display()))?;
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

    let report = VcfSchemaValidationReport {
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
    };

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&output_path, &report)
        .with_context(|| format!("write {}", output_path.display()))?;

    if !report.passes_gate {
        bail!(
            "VCF schema validation failed for `{}` and `{}`",
            report.shared_schema_path,
            report.stage_dir
        );
    }

    Ok(report)
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
