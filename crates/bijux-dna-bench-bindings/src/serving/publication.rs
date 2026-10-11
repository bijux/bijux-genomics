use super::{service::collect, tsv::render_tool_serving_map_tsv, ToolServingMapReport};
use crate::{
    observations::Domain,
    publication::{path_relative_to_repo, repo_relative_path},
    ServingContext,
};
use anyhow::Result;
use std::path::Path;
/// Construct the declared evidence from explicit inputs.
///
/// # Errors
/// Returns acquisition, semantic admission or publication failure.
pub fn render_fastq_tool_serving_map(
    context: &ServingContext<'_>,
    output_path: &Path,
) -> Result<ToolServingMapReport> {
    let repo_root = context.repo_root;
    let output_path = repo_relative_path(repo_root, output_path);
    let observed = collect(context, Domain::Fastq)?;
    let stage_count = observed.stage_count;
    let tool_count = observed.tool_count;
    let rows = observed.rows;

    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_tool_serving_map_tsv(&rows).as_bytes(),
    )?;

    Ok(ToolServingMapReport {
        schema_version: FASTQ_TOOL_SERVING_MAP_SCHEMA_VERSION,
        domain: "fastq",
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        tool_count,
        row_count: rows.len(),
        rows,
    })
}
/// Construct the declared evidence from explicit inputs.
///
/// # Errors
/// Returns acquisition, semantic admission or publication failure.
pub fn render_bam_tool_serving_map(
    context: &ServingContext<'_>,
    output_path: &Path,
) -> Result<ToolServingMapReport> {
    let repo_root = context.repo_root;
    let output_path = repo_relative_path(repo_root, output_path);
    let observed = collect(context, Domain::Bam)?;
    let stage_count = observed.stage_count;
    let tool_count = observed.tool_count;
    let rows = observed.rows;

    bijux_dna_infra::atomic_write_bytes(
        &output_path,
        render_tool_serving_map_tsv(&rows).as_bytes(),
    )?;

    Ok(ToolServingMapReport {
        schema_version: BAM_TOOL_SERVING_MAP_SCHEMA_VERSION,
        domain: "bam",
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        tool_count,
        row_count: rows.len(),
        rows,
    })
}

use super::{BAM_TOOL_SERVING_MAP_SCHEMA_VERSION, FASTQ_TOOL_SERVING_MAP_SCHEMA_VERSION};
