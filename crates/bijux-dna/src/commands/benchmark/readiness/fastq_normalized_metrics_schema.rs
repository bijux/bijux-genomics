use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::schema_paths::DEFAULT_FASTQ_NORMALIZED_METRICS_SCHEMA_PATH;
use crate::commands::cli::parse;
use crate::commands::cli::render;

const FASTQ_NORMALIZED_METRICS_SCHEMA_REPORT_VERSION: &str =
    "bijux.bench.readiness.fastq_normalized_metrics_schema.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FastqNormalizedMetricsSchemaRow {
    pub(crate) stage_id: String,
    pub(crate) extension_id: String,
    pub(crate) required_key_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqNormalizedMetricsSchemaReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) schema_id: String,
    pub(crate) stage_count: usize,
    pub(crate) extension_count: usize,
    pub(crate) rows: Vec<FastqNormalizedMetricsSchemaRow>,
}

pub(crate) fn run_render_fastq_normalized_metrics_schema(
    args: &parse::BenchReadinessRenderFastqNormalizedMetricsSchemaArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_normalized_metrics_schema(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_NORMALIZED_METRICS_SCHEMA_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_fastq_normalized_metrics_schema(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqNormalizedMetricsSchemaReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_api::v1::api::bench::write_fastq_normalized_metrics_schema(&output_path)?;

    let schema = bijux_dna_api::v1::api::bench::render_fastq_normalized_metrics_schema();
    let schema_id = schema
        .get("$id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("FASTQ normalized metrics schema is missing string `$id`"))?
        .to_string();
    let rows = collect_fastq_normalized_metrics_schema_rows(&schema)?;

    Ok(FastqNormalizedMetricsSchemaReport {
        schema_version: FASTQ_NORMALIZED_METRICS_SCHEMA_REPORT_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        schema_id,
        stage_count: rows.len(),
        extension_count: rows.len(),
        rows,
    })
}

fn collect_fastq_normalized_metrics_schema_rows(
    schema: &serde_json::Value,
) -> Result<Vec<FastqNormalizedMetricsSchemaRow>> {
    let stage_defs = schema
        .get("$defs")
        .and_then(|value| value.get("stages"))
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            anyhow!("FASTQ normalized metrics schema is missing object `$defs.stages`")
        })?;

    let mut rows = stage_defs
        .iter()
        .map(|(stage_id, value)| {
            let stage_contract = value
                .get("allOf")
                .and_then(serde_json::Value::as_array)
                .and_then(|items| items.get(1))
                .ok_or_else(|| anyhow!("FASTQ normalized metrics stage `{stage_id}` is missing stage extension"))?;
            let extension_id = stage_contract
                .get("x-bijux-extension-id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| {
                    anyhow!(
                        "FASTQ normalized metrics stage `{stage_id}` is missing string `x-bijux-extension-id`"
                    )
                })?;
            let required_key_count = stage_contract
                .get("required")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| {
                    anyhow!(
                        "FASTQ normalized metrics stage `{stage_id}` is missing `required` keys"
                    )
                })?
                .len();
            Ok(FastqNormalizedMetricsSchemaRow {
                stage_id: stage_id.clone(),
                extension_id: extension_id.to_string(),
                required_key_count,
            })
        })
        .collect::<Result<Vec<_>>>()?;

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
