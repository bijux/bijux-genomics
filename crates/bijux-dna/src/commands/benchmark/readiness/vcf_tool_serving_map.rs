use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_vcf_stage_catalog::{
    build_vcf_stage_catalog_rows, VcfStageCatalogRow,
};
use crate::commands::benchmark::local_vcf_stage_matrix::VcfStageMatrixRow;
use crate::commands::benchmark::vcf_benchmark_bindings::collect_vcf_benchmark_binding_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_TOOL_SERVING_MAP_PATH: &str =
    "benchmarks/readiness/vcf-tool-serving-map.tsv";
const VCF_TOOL_SERVING_MAP_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_tool_serving_map.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfToolServingMapRow {
    pub(crate) tool_id: String,
    pub(crate) stage_id: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
    pub(crate) asset_status: String,
    pub(crate) benchmark_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfToolServingMapReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) not_benchmark_ready_row_count: usize,
    pub(crate) rows: Vec<VcfToolServingMapRow>,
}

pub(crate) fn run_render_vcf_tool_serving_map(
    args: &parse::BenchReadinessRenderVcfToolServingMapArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_tool_serving_map(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_TOOL_SERVING_MAP_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_tool_serving_map(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfToolServingMapReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_vcf_tool_serving_map_rows()?;
    let row_count = rows.len();
    let stage_count = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let not_benchmark_ready_row_count = row_count.saturating_sub(benchmark_ready_row_count);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_vcf_tool_serving_map_tsv(&rows))
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

pub(crate) fn collect_vcf_tool_serving_map_rows() -> Result<Vec<VcfToolServingMapRow>> {
    let catalog_rows = build_vcf_stage_catalog_rows()?;
    let catalog_by_stage =
        catalog_rows.into_iter().map(|row| (row.stage_id.clone(), row)).collect::<BTreeMap<_, _>>();
    let matrix_rows = collect_vcf_benchmark_binding_rows()?;

    let mut rows = Vec::with_capacity(matrix_rows.len());
    for matrix_row in matrix_rows {
        let catalog_row = catalog_by_stage.get(matrix_row.stage_id.as_str()).ok_or_else(|| {
            anyhow!(
                "VCF tool-serving map is missing stage catalog coverage for `{}`",
                matrix_row.stage_id
            )
        })?;
        rows.push(build_tool_serving_row(&matrix_row, catalog_row)?);
    }

    rows.sort_by(|left, right| {
        left.tool_id.cmp(&right.tool_id).then_with(|| left.stage_id.cmp(&right.stage_id))
    });
    ensure_vcf_tool_serving_map_contract(&rows)?;
    Ok(rows)
}

fn build_tool_serving_row(
    matrix_row: &VcfStageMatrixRow,
    catalog_row: &VcfStageCatalogRow,
) -> Result<VcfToolServingMapRow> {
    let support_status = catalog_row.support_status.clone();
    let adapter_status = adapter_status_label(&support_status, &matrix_row.adapter_id);
    let parser_status = parser_status_label(&matrix_row.parser_id)?;
    let corpus_status = corpus_status_label(&matrix_row.corpus_id);
    let asset_status =
        asset_status_label(&matrix_row.asset_profile_id, &catalog_row.required_assets)?;
    let benchmark_status = benchmark_status_label(&support_status);

    Ok(VcfToolServingMapRow {
        tool_id: matrix_row.tool_id.clone(),
        stage_id: matrix_row.stage_id.clone(),
        support_status,
        adapter_status,
        parser_status,
        corpus_status,
        asset_status,
        benchmark_status,
    })
}

fn adapter_status_label(support_status: &str, adapter_id: &str) -> String {
    if adapter_id.trim().is_empty() {
        "missing".to_string()
    } else if support_status == "supported" {
        "runnable".to_string()
    } else {
        "declared_only".to_string()
    }
}

fn parser_status_label(parser_id: &str) -> Result<String> {
    let label = match parser_id {
        "vcf.parser.vcf_output"
        | "vcf.parser.call_summary"
        | "vcf.parser.qc_report"
        | "vcf.parser.report_json"
        | "vcf.parser.stats_report" => "parse_normalized",
        other => {
            return Err(anyhow!(
                "VCF tool-serving map encountered unknown parser contract `{other}`"
            ));
        }
    };
    Ok(label.to_string())
}

fn corpus_status_label(corpus_id: &str) -> String {
    format!("fixture:{corpus_id}")
}

fn asset_status_label(asset_profile_id: &str, required_assets: &[String]) -> Result<String> {
    if asset_profile_id.trim().is_empty() {
        return Err(anyhow!("VCF tool-serving map encountered an empty asset profile id"));
    }
    if required_assets.is_empty() {
        Ok("not_required".to_string())
    } else {
        Ok("assigned".to_string())
    }
}

fn benchmark_status_label(support_status: &str) -> String {
    match support_status {
        "supported" => "benchmark_ready",
        "planned" => "not_benchmark_ready",
        _ => "not_benchmark_ready",
    }
    .to_string()
}

fn ensure_vcf_tool_serving_map_contract(rows: &[VcfToolServingMapRow]) -> Result<()> {
    let expected_rows = [
        (
            "bcftools",
            "vcf.call",
            "supported",
            "runnable",
            "parse_normalized",
            "fixture:vcf_production_regression",
            "assigned",
            "benchmark_ready",
        ),
        (
            "bcftools",
            "vcf.postprocess",
            "supported",
            "runnable",
            "parse_normalized",
            "fixture:vcf_production_regression",
            "assigned",
            "benchmark_ready",
        ),
        (
            "bcftools",
            "vcf.prepare_reference_panel",
            "supported",
            "runnable",
            "parse_normalized",
            "fixture:vcf_production_regression",
            "assigned",
            "benchmark_ready",
        ),
        (
            "bcftools",
            "vcf.qc",
            "supported",
            "runnable",
            "parse_normalized",
            "fixture:vcf_production_regression",
            "assigned",
            "benchmark_ready",
        ),
        (
            "plink",
            "vcf.qc",
            "supported",
            "runnable",
            "parse_normalized",
            "fixture:vcf_production_regression",
            "assigned",
            "benchmark_ready",
        ),
        (
            "plink2",
            "vcf.qc",
            "supported",
            "runnable",
            "parse_normalized",
            "fixture:vcf_production_regression",
            "assigned",
            "benchmark_ready",
        ),
        (
            "ibdne",
            "vcf.demography",
            "planned",
            "declared_only",
            "parse_normalized",
            "fixture:vcf_production_regression",
            "not_required",
            "not_benchmark_ready",
        ),
    ];

    for (
        tool_id,
        stage_id,
        support_status,
        adapter_status,
        parser_status,
        corpus_status,
        asset_status,
        benchmark_status,
    ) in expected_rows
    {
        let row = rows
            .iter()
            .find(|row| row.tool_id == tool_id && row.stage_id == stage_id)
            .ok_or_else(|| anyhow!("VCF tool-serving map is missing `{stage_id}` / `{tool_id}`"))?;
        if row.support_status != support_status
            || row.adapter_status != adapter_status
            || row.parser_status != parser_status
            || row.corpus_status != corpus_status
            || row.asset_status != asset_status
            || row.benchmark_status != benchmark_status
        {
            return Err(anyhow!(
                "VCF tool-serving row `{}` / `{}` drifted from its governed readiness contract",
                stage_id,
                tool_id
            ));
        }
    }

    let expected_row_count = collect_vcf_benchmark_binding_rows()?.len();
    if rows.len() != expected_row_count {
        return Err(anyhow!(
            "VCF tool-serving map must contain exactly one row per matrix row (expected {}, found {})",
            expected_row_count,
            rows.len()
        ));
    }

    let mut seen_bindings = BTreeSet::<(&str, &str)>::new();
    for row in rows {
        if !seen_bindings.insert((row.stage_id.as_str(), row.tool_id.as_str())) {
            return Err(anyhow!(
                "VCF tool-serving map contains duplicate row `{}` / `{}`",
                row.stage_id,
                row.tool_id
            ));
        }
    }

    Ok(())
}

fn render_vcf_tool_serving_map_tsv(rows: &[VcfToolServingMapRow]) -> String {
    let mut rendered = String::from(
        "tool_id\tstage_id\tsupport_status\tadapter_status\tparser_status\tcorpus_status\tasset_status\tbenchmark_status\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.corpus_status),
            sanitize_tsv(&row.asset_status),
            sanitize_tsv(&row.benchmark_status),
        ));
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_vcf_tool_serving_map, DEFAULT_VCF_TOOL_SERVING_MAP_PATH,
        VCF_TOOL_SERVING_MAP_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_tool_serving_map_tracks_owned_matrix_rows() {
        let root = repo_root();
        let report =
            render_vcf_tool_serving_map(&root, PathBuf::from(DEFAULT_VCF_TOOL_SERVING_MAP_PATH))
                .expect("render VCF tool serving map");

        assert_eq!(report.schema_version, VCF_TOOL_SERVING_MAP_SCHEMA_VERSION);
        assert_eq!(report.domain, "vcf");
        assert_eq!(report.row_count, 22);
        assert_eq!(report.stage_count, 20);
        assert_eq!(report.tool_count, 7);
        assert_eq!(report.benchmark_ready_row_count, 14);
        assert_eq!(report.not_benchmark_ready_row_count, 8);
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bcftools"
                && row.stage_id == "vcf.call"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parse_normalized"
                && row.corpus_status == "fixture:vcf_production_regression"
                && row.asset_status == "assigned"
                && row.benchmark_status == "benchmark_ready"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bcftools"
                && row.stage_id == "vcf.qc"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parse_normalized"
                && row.corpus_status == "fixture:vcf_production_regression"
                && row.asset_status == "assigned"
                && row.benchmark_status == "benchmark_ready"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "plink"
                && row.stage_id == "vcf.qc"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parse_normalized"
                && row.corpus_status == "fixture:vcf_production_regression"
                && row.asset_status == "assigned"
                && row.benchmark_status == "benchmark_ready"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bcftools"
                && row.stage_id == "vcf.prepare_reference_panel"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parse_normalized"
                && row.corpus_status == "fixture:vcf_production_regression"
                && row.asset_status == "assigned"
                && row.benchmark_status == "benchmark_ready"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "shapeit5"
                && row.stage_id == "vcf.phasing"
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parse_normalized"
                && row.corpus_status == "fixture:vcf_production_regression"
                && row.asset_status == "assigned"
                && row.benchmark_status == "benchmark_ready"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "ibdne"
                && row.stage_id == "vcf.demography"
                && row.support_status == "planned"
                && row.adapter_status == "declared_only"
                && row.parser_status == "parse_normalized"
                && row.corpus_status == "fixture:vcf_production_regression"
                && row.asset_status == "not_required"
                && row.benchmark_status == "not_benchmark_ready"
        }));
    }
}
