use super::{
    collect_vcf_tool_serving_map_rows, VcfToolServingMapReport, VcfToolServingMapRow,
    VCF_TOOL_SERVING_MAP_SCHEMA_VERSION,
};
use crate::inputs::VcfAdapterInputs;
use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Publish the compatible serving report after admitted-input projection succeeds.
///
/// # Errors
/// Returns projection or filesystem publication errors.
pub fn render_vcf_tool_serving_map(
    inputs: &VcfAdapterInputs,
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfToolServingMapReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let rows = collect_vcf_tool_serving_map_rows(inputs)?;
    let row_count = rows.len();
    let stage_count = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let not_benchmark_ready_row_count = row_count.saturating_sub(benchmark_ready_row_count);

    let tsv = render_vcf_tool_serving_map_tsv(&rows);
    bijux_dna_infra::atomic_write_bytes(&output_path, tsv.as_bytes())
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(VcfToolServingMapReport {
        schema_version: VCF_TOOL_SERVING_MAP_SCHEMA_VERSION,
        domain: "vcf",
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count,
        stage_count,
        tool_count,
        benchmark_ready_row_count,
        not_benchmark_ready_row_count,
        rows,
    })
}

fn render_vcf_tool_serving_map_tsv(rows: &[VcfToolServingMapRow]) -> String {
    let mut rendered = String::from(
        "tool_id\tstage_id\tsupport_status\tadapter_status\tparser_status\tcorpus_status\tasset_status\tbenchmark_status\n",
    );
    for row in rows {
        let columns = [
            &row.tool_id,
            &row.stage_id,
            &row.support_status,
            &row.adapter_status,
            &row.parser_status,
            &row.corpus_status,
            &row.asset_status,
            &row.benchmark_status,
        ];
        rendered.push_str(
            &columns.into_iter().map(|value| sanitize_tsv(value)).collect::<Vec<_>>().join("\t"),
        );
        rendered.push('\n');
    }
    rendered
}

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
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
