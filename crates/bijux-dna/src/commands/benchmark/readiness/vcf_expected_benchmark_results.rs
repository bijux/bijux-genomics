use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::contracts::stage_metrics_contract;
use bijux_dna_domain_vcf::VcfDomainStage;
use serde::Serialize;

use super::vcf_bcftools_adapter::collect_vcf_bcftools_adapter_rows;
use super::vcf_plink_family_adapter::collect_vcf_plink_family_adapter_rows_for_tool;
use super::vcf_tool_serving_map::collect_vcf_tool_serving_map_rows;
use crate::commands::benchmark::local_vcf_stage_catalog::{
    build_vcf_stage_catalog_rows, VcfStageCatalogRow,
};
use crate::commands::benchmark::local_vcf_stage_matrix::VcfStageMatrixRow;
use crate::commands::benchmark::vcf_benchmark_bindings::collect_vcf_benchmark_binding_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH: &str =
    "benchmarks/readiness/vcf-expected-benchmark-results.tsv";
const VCF_EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_expected_benchmark_results.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfExpectedBenchmarkResultRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) expected_outputs: Vec<String>,
    pub(crate) expected_metrics: Vec<String>,
    pub(crate) report_section: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfExpectedBenchmarkResultsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) corpus_count: usize,
    pub(crate) asset_profile_count: usize,
    pub(crate) report_section_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<VcfExpectedBenchmarkResultRow>,
}

pub(crate) fn run_render_vcf_expected_benchmark_results(
    args: &parse::BenchReadinessRenderVcfExpectedBenchmarkResultsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_expected_benchmark_results(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_expected_benchmark_results(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfExpectedBenchmarkResultsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_vcf_expected_benchmark_result_rows(repo_root)?;
    let stage_count = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let corpus_count = rows.iter().map(|row| row.corpus_id.as_str()).collect::<BTreeSet<_>>().len();
    let asset_profile_count =
        rows.iter().map(|row| row.asset_profile_id.as_str()).collect::<BTreeSet<_>>().len();
    let mut report_section_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *report_section_counts.entry(row.report_section.clone()).or_default() += 1;
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_vcf_expected_benchmark_results_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(VcfExpectedBenchmarkResultsReport {
        schema_version: VCF_EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        stage_count,
        tool_count,
        corpus_count,
        asset_profile_count,
        report_section_counts,
        rows,
    })
}

pub(crate) fn collect_vcf_expected_benchmark_result_rows(
    repo_root: &Path,
) -> Result<Vec<VcfExpectedBenchmarkResultRow>> {
    let benchmark_ready_rows = collect_vcf_tool_serving_map_rows()?
        .into_iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .collect::<Vec<_>>();
    let matrix_by_pair = collect_vcf_benchmark_binding_rows()?
        .into_iter()
        .map(|row| ((row.stage_id.clone(), row.tool_id.clone()), row))
        .collect::<BTreeMap<_, _>>();
    let catalog_by_stage = build_vcf_stage_catalog_rows()?
        .into_iter()
        .map(|row| (row.stage_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let adapter_rows = collect_vcf_bcftools_adapter_rows(repo_root)?
        .into_iter()
        .map(|row| ((row.stage_id.clone(), row.tool_id.clone()), row.into()))
        .chain(
            collect_vcf_plink_family_adapter_rows_for_tool(repo_root, "plink")?
                .into_iter()
                .map(|row| ((row.stage_id.clone(), row.tool_id.clone()), row.into())),
        )
        .chain(
            collect_vcf_plink_family_adapter_rows_for_tool(repo_root, "plink2")?
                .into_iter()
                .map(|row| ((row.stage_id.clone(), row.tool_id.clone()), row.into())),
        )
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::with_capacity(benchmark_ready_rows.len());
    for serving_row in benchmark_ready_rows {
        let pair_key = (serving_row.stage_id.clone(), serving_row.tool_id.clone());
        let matrix_row = matrix_by_pair.get(&pair_key).ok_or_else(|| {
            anyhow!(
                "VCF expected-result table is missing matrix coverage for `{}` / `{}`",
                serving_row.stage_id,
                serving_row.tool_id
            )
        })?;
        let catalog_row = catalog_by_stage.get(serving_row.stage_id.as_str()).ok_or_else(|| {
            anyhow!(
                "VCF expected-result table is missing stage catalog coverage for `{}`",
                serving_row.stage_id
            )
        })?;
        let stage = VcfDomainStage::try_from(serving_row.stage_id.as_str())
            .map_err(|error| anyhow!("unknown VCF stage `{}`: {error}", serving_row.stage_id))?;
        let expected_metrics = stage_metrics_contract(stage)
            .required_metrics
            .iter()
            .map(|metric| (*metric).to_string())
            .collect::<Vec<_>>();
        if expected_metrics.is_empty() {
            return Err(anyhow!(
                "VCF expected-result table stage `{}` is missing required metrics",
                serving_row.stage_id
            ));
        }
        let expected_outputs = expected_outputs_for_binding(
            &serving_row.tool_id,
            &serving_row.stage_id,
            matrix_row,
            &adapter_rows,
        )?;
        let report_section = report_section_for_stage(catalog_row)?;

        rows.push(VcfExpectedBenchmarkResultRow {
            domain: "vcf".to_string(),
            stage_id: serving_row.stage_id,
            tool_id: serving_row.tool_id,
            corpus_id: matrix_row.corpus_id.clone(),
            asset_profile_id: matrix_row.asset_profile_id.clone(),
            expected_outputs,
            expected_metrics,
            report_section,
        });
    }

    rows.sort_by(|left, right| {
        left.stage_id
            .cmp(&right.stage_id)
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.corpus_id.cmp(&right.corpus_id))
            .then_with(|| left.asset_profile_id.cmp(&right.asset_profile_id))
    });
    ensure_vcf_expected_benchmark_result_contract(&rows)?;
    Ok(rows)
}

fn expected_outputs_for_binding(
    tool_id: &str,
    stage_id: &str,
    matrix_row: &VcfStageMatrixRow,
    adapter_rows: &BTreeMap<(String, String), VcfExpectedOutputsAdapterRow>,
) -> Result<Vec<String>> {
    let adapter_row = adapter_rows
        .get(&(stage_id.to_string(), tool_id.to_string()))
        .ok_or_else(|| {
            anyhow!(
                "VCF expected-result table is missing adapter coverage for `{stage_id}` / `{tool_id}`"
            )
        })?;
    let expected_outputs = adapter_row.stage_output_ids.clone();
    if expected_outputs != matrix_row.expected_outputs {
        return Err(anyhow!(
            "VCF expected-result table stage output drifted for `{stage_id}` / `{tool_id}`"
        ));
    }
    if expected_outputs.is_empty() {
        return Err(anyhow!(
            "VCF expected-result table row `{stage_id}` / `{tool_id}` is missing expected outputs"
        ));
    }
    Ok(expected_outputs)
}

#[derive(Debug, Clone)]
struct VcfExpectedOutputsAdapterRow {
    stage_output_ids: Vec<String>,
}

impl From<super::vcf_bcftools_adapter::VcfBcftoolsAdapterRow> for VcfExpectedOutputsAdapterRow {
    fn from(value: super::vcf_bcftools_adapter::VcfBcftoolsAdapterRow) -> Self {
        Self { stage_output_ids: value.stage_output_ids }
    }
}

impl From<super::vcf_plink_family_adapter::VcfPlinkFamilyAdapterRow>
    for VcfExpectedOutputsAdapterRow
{
    fn from(value: super::vcf_plink_family_adapter::VcfPlinkFamilyAdapterRow) -> Self {
        Self { stage_output_ids: value.stage_output_ids }
    }
}

fn report_section_for_stage(catalog_row: &VcfStageCatalogRow) -> Result<String> {
    if catalog_row.benchmark_category.trim().is_empty() {
        return Err(anyhow!(
            "VCF expected-result table stage `{}` is missing benchmark_category",
            catalog_row.stage_id
        ));
    }
    Ok(catalog_row.benchmark_category.clone())
}

fn render_vcf_expected_benchmark_results_tsv(rows: &[VcfExpectedBenchmarkResultRow]) -> String {
    let mut rendered = String::from(
        "domain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\texpected_outputs\texpected_metrics\treport_section\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.corpus_id),
            sanitize_tsv(&row.asset_profile_id),
            sanitize_tsv(&row.expected_outputs.join(",")),
            sanitize_tsv(&row.expected_metrics.join(",")),
            sanitize_tsv(&row.report_section),
        ));
    }
    rendered
}

fn ensure_vcf_expected_benchmark_result_contract(
    rows: &[VcfExpectedBenchmarkResultRow],
) -> Result<()> {
    let unique_rows = rows
        .iter()
        .map(|row| {
            format!("{}:{}:{}:{}", row.stage_id, row.tool_id, row.corpus_id, row.asset_profile_id)
        })
        .collect::<BTreeSet<_>>();
    if unique_rows.len() != rows.len() {
        return Err(anyhow!(
            "VCF expected-result table must keep one row per benchmark-ready stage-tool-corpus-asset binding"
        ));
    }
    if rows.len() != 12 {
        return Err(anyhow!(
            "VCF expected-result table must retain exactly 12 benchmark-ready rows, found {}",
            rows.len()
        ));
    }
    if rows.iter().any(|row| row.domain != "vcf") {
        return Err(anyhow!(
            "VCF expected-result table must keep the VCF domain label on every row"
        ));
    }
    for (
        stage_id,
        tool_id,
        corpus_id,
        asset_profile_id,
        expected_outputs,
        expected_metrics,
        report_section,
    ) in [
        (
            "vcf.qc",
            "bcftools",
            "vcf_production_regression",
            "vcf_cohort",
            "qc_report",
            "hwe_summary",
            "quality_control",
        ),
        (
            "vcf.call",
            "bcftools",
            "vcf_production_regression",
            "bam_bundle",
            "called_vcf",
            "variant_count",
            "variant_calling",
        ),
        (
            "vcf.gl_propagation",
            "bcftools",
            "vcf_production_regression",
            "vcf_single_sample",
            "gl_propagated_vcf",
            "lost_fields",
            "likelihood_postprocess",
        ),
        (
            "vcf.postprocess",
            "bcftools",
            "vcf_production_regression",
            "vcf_single_sample",
            "postprocess_vcf",
            "readable_vcf",
            "normalization",
        ),
        (
            "vcf.stats",
            "bcftools",
            "vcf_production_regression",
            "vcf_cohort",
            "stats_json",
            "ti_tv",
            "quality_control",
        ),
    ] {
        let row =
            rows.iter().find(|row| row.stage_id == stage_id && row.tool_id == tool_id).ok_or_else(
                || anyhow!("VCF expected-result table is missing `{stage_id}` / `{tool_id}`"),
            )?;
        if row.corpus_id != corpus_id
            || row.asset_profile_id != asset_profile_id
            || !row.expected_outputs.iter().any(|candidate| candidate == expected_outputs)
            || !row.expected_metrics.iter().any(|candidate| candidate == expected_metrics)
            || row.report_section != report_section
        {
            return Err(anyhow!(
                "VCF expected-result table row `{stage_id}` / `{tool_id}` drifted from the governed corpus, asset, output, metrics, or section contract"
            ));
        }
    }
    Ok(())
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

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_vcf_expected_benchmark_results, DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH,
        VCF_EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_vcf_expected_benchmark_results_tracks_benchmark_ready_rows() {
        let root = repo_root();
        let report = render_vcf_expected_benchmark_results(
            &root,
            PathBuf::from(DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH),
        )
        .expect("render VCF expected benchmark results");

        assert_eq!(report.schema_version, VCF_EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH);
        assert_eq!(report.row_count, 12);
        assert_eq!(report.stage_count, 10);
        assert_eq!(report.tool_count, 3);
        assert_eq!(report.corpus_count, 1);
        assert_eq!(report.asset_profile_count, 3);
        assert_eq!(report.report_section_counts.get("variant_calling"), Some(&4));
        assert_eq!(report.report_section_counts.get("quality_control"), Some(&5));
        assert_eq!(report.report_section_counts.get("normalization"), Some(&1));

        assert!(report.rows.iter().all(|row| {
            row.domain == "vcf"
                && row.corpus_id == "vcf_production_regression"
                && !row.expected_outputs.is_empty()
                && !row.expected_metrics.is_empty()
                && !row.report_section.is_empty()
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.call"
                && row.asset_profile_id == "bam_bundle"
                && row.expected_outputs == vec!["called_vcf".to_string()]
                && row.expected_metrics.iter().any(|metric| metric == "variant_count")
                && row.report_section == "variant_calling"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.gl_propagation"
                && row.asset_profile_id == "vcf_single_sample"
                && row.expected_outputs == vec!["gl_propagated_vcf".to_string()]
                && row.expected_metrics.iter().any(|metric| metric == "lost_fields")
                && row.report_section == "likelihood_postprocess"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.stats"
                && row.asset_profile_id == "vcf_cohort"
                && row.expected_outputs == vec!["stats_json".to_string()]
                && row.expected_metrics.iter().any(|metric| metric == "ti_tv")
                && row.report_section == "quality_control"
        }));
    }
}
