use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::schema_paths::{
    DEFAULT_VCF_NORMALIZED_METRICS_SCHEMA_PATH, DEFAULT_VCF_NORMALIZED_METRICS_STAGE_DIR,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const VCF_NORMALIZED_METRICS_SCHEMA_REPORT_VERSION: &str =
    "bijux.bench.readiness.vcf_normalized_metrics_schema.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfNormalizedMetricsSchemaRow {
    pub(crate) stage_id: String,
    pub(crate) schema_version: String,
    pub(crate) schema_id: String,
    pub(crate) file_name: String,
    pub(crate) extension_id: String,
    pub(crate) required_key_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfNormalizedMetricsSchemaReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_dir: String,
    pub(crate) schema_id: String,
    pub(crate) stage_count: usize,
    pub(crate) extension_count: usize,
    pub(crate) rows: Vec<VcfNormalizedMetricsSchemaRow>,
}

pub(crate) fn run_render_vcf_normalized_metrics_schema(
    args: &parse::BenchReadinessRenderVcfNormalizedMetricsSchemaArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_normalized_metrics_schema(
        &repo_root,
        args.output
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

pub(crate) fn render_vcf_normalized_metrics_schema(
    repo_root: &Path,
    output_path: PathBuf,
    stage_dir: PathBuf,
) -> Result<VcfNormalizedMetricsSchemaReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let stage_dir = repo_relative_path(repo_root, &stage_dir);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::create_dir_all(&stage_dir).with_context(|| format!("create {}", stage_dir.display()))?;

    bijux_dna_api::v1::api::bench::write_vcf_normalized_metrics_schema(&output_path)?;
    let descriptors =
        bijux_dna_api::v1::api::bench::write_vcf_normalized_metrics_stage_schemas(&stage_dir)?;
    let schema = bijux_dna_api::v1::api::bench::render_vcf_normalized_metrics_schema()?;
    let schema_id = schema
        .get("$id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("VCF normalized metrics schema is missing string `$id`"))?
        .to_string();

    let mut rows = descriptors
        .into_iter()
        .map(|descriptor| VcfNormalizedMetricsSchemaRow {
            stage_id: descriptor.stage_id,
            schema_version: descriptor.schema_version,
            schema_id: descriptor.schema_id,
            file_name: descriptor.file_name,
            extension_id: descriptor.extension_id,
            required_key_count: descriptor.required_key_count,
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));

    Ok(VcfNormalizedMetricsSchemaReport {
        schema_version: VCF_NORMALIZED_METRICS_SCHEMA_REPORT_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_dir: path_relative_to_repo(repo_root, &stage_dir),
        schema_id,
        stage_count: rows.len(),
        extension_count: rows.len(),
        rows,
    })
}

pub(crate) fn collect_vcf_normalized_metrics_schema_report_rows(
) -> Result<Vec<VcfNormalizedMetricsSchemaRow>> {
    let mut rows =
        bijux_dna_api::v1::api::bench::vcf_normalized_metrics_stage_schema_descriptors()?
            .into_iter()
            .map(|descriptor| VcfNormalizedMetricsSchemaRow {
                stage_id: descriptor.stage_id,
                schema_version: descriptor.schema_version,
                schema_id: descriptor.schema_id,
                file_name: descriptor.file_name,
                extension_id: descriptor.extension_id,
                required_key_count: descriptor.required_key_count,
            })
            .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    Ok(rows)
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
