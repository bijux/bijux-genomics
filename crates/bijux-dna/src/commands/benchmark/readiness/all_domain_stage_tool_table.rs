use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::bam_command_adapter_coverage::{
    collect_bam_command_adapter_coverage_rows, BamBenchmarkStatus,
};
use super::bam_corpus_assignment::collect_bam_corpus_assignment_rows;
use super::fastq_command_adapter_coverage::{
    collect_fastq_command_adapter_coverage_rows, FastqBenchmarkStatus,
};
use super::fastq_corpus_assignment::collect_fastq_corpus_assignment_rows;
use super::stage_tool_assets::{
    StageToolAssetRow, StageToolAssetsConfig, DEFAULT_STAGE_TOOL_ASSETS_PATH,
    LOCAL_STAGE_TOOL_ASSETS_SCHEMA_VERSION,
};
use super::vcf_tool_serving_map::collect_vcf_tool_serving_map_rows;
use crate::commands::benchmark::vcf_benchmark_bindings::collect_vcf_benchmark_binding_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH: &str =
    "benchmarks/readiness/all-domain-stage-tool-table.tsv";
const ALL_DOMAIN_STAGE_TOOL_TABLE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_stage_tool_table.v1";
const NOT_ASSIGNED: &str = "not_assigned";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    domain: String,
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainStageToolTableRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) benchmark_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainStageToolTableReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) benchmark_ready_unique_binding_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) benchmark_ready_domain_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<AllDomainStageToolTableRow>,
}

pub(crate) fn run_render_all_domain_stage_tool_table(
    args: &parse::BenchReadinessRenderAllDomainStageToolTableArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_stage_tool_table(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_stage_tool_table(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainStageToolTableReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_all_domain_stage_tool_table_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_all_domain_stage_tool_table_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let row_count = rows.len();
    let benchmark_ready_rows =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").collect::<Vec<_>>();
    let benchmark_ready_row_count = benchmark_ready_rows.len();
    let benchmark_ready_unique_binding_count = benchmark_ready_rows
        .iter()
        .map(|row| binding_key(&row.domain, &row.stage_id, &row.tool_id))
        .collect::<BTreeSet<_>>()
        .len();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut benchmark_ready_domain_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        if row.benchmark_status == "benchmark_ready" {
            *benchmark_ready_domain_counts.entry(row.domain.clone()).or_default() += 1;
        }
    }

    Ok(AllDomainStageToolTableReport {
        schema_version: ALL_DOMAIN_STAGE_TOOL_TABLE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count,
        benchmark_ready_row_count,
        benchmark_ready_unique_binding_count,
        domain_counts,
        benchmark_ready_domain_counts,
        rows,
    })
}

pub(crate) fn collect_all_domain_stage_tool_table_rows(
    repo_root: &Path,
) -> Result<Vec<AllDomainStageToolTableRow>> {
    let asset_roles_by_binding = load_committed_stage_tool_asset_rows(repo_root)?.into_iter().fold(
        BTreeMap::<BindingKey, Vec<String>>::new(),
        |mut acc, row| {
            acc.entry(binding_key(&row.domain, &row.stage_id, &row.tool_id))
                .or_default()
                .push(row.asset_role);
            acc
        },
    );

    let fastq_corpus_rows = collect_fastq_corpus_assignment_rows(repo_root)?
        .2
        .into_iter()
        .map(|row| (binding_key("fastq", &row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();
    let bam_corpus_rows = collect_bam_corpus_assignment_rows(repo_root)?
        .2
        .into_iter()
        .map(|row| (binding_key("bam", &row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();
    let vcf_matrix_rows = collect_vcf_benchmark_binding_rows()?
        .into_iter()
        .map(|row| (binding_key("vcf", &row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::new();

    for row in collect_fastq_command_adapter_coverage_rows(repo_root)?.2 {
        let key = binding_key("fastq", &row.stage_id, &row.tool_id);
        let corpus_row = fastq_corpus_rows.get(&key).ok_or_else(|| {
            anyhow!(
                "all-domain stage-tool table is missing FASTQ corpus assignment for `{}` / `{}`",
                row.stage_id,
                row.tool_id
            )
        })?;
        rows.push(AllDomainStageToolTableRow {
            domain: "fastq".to_string(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            corpus_id: corpus_row
                .fixture_id
                .clone()
                .or(corpus_row.benchmark_scope_id.clone())
                .unwrap_or_else(|| NOT_ASSIGNED.to_string()),
            asset_profile_id: fastq_bam_asset_profile_id(&key, &asset_roles_by_binding),
            adapter_id: normalized_adapter_surface_id("fastq", &row.stage_id),
            parser_id: normalized_parser_surface_id("fastq", &row.stage_id),
            benchmark_status: fastq_benchmark_status_label(row.benchmark_status).to_string(),
        });
    }

    for row in collect_bam_command_adapter_coverage_rows(repo_root)?.2 {
        let key = binding_key("bam", &row.stage_id, &row.tool_id);
        let corpus_row = bam_corpus_rows.get(&key).ok_or_else(|| {
            anyhow!(
                "all-domain stage-tool table is missing BAM corpus assignment for `{}` / `{}`",
                row.stage_id,
                row.tool_id
            )
        })?;
        rows.push(AllDomainStageToolTableRow {
            domain: "bam".to_string(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            corpus_id: corpus_row.fixture_id.clone(),
            asset_profile_id: fastq_bam_asset_profile_id(&key, &asset_roles_by_binding),
            adapter_id: normalized_adapter_surface_id("bam", &row.stage_id),
            parser_id: normalized_parser_surface_id("bam", &row.stage_id),
            benchmark_status: bam_benchmark_status_label(row.benchmark_status).to_string(),
        });
    }

    for row in collect_vcf_tool_serving_map_rows()? {
        let key = binding_key("vcf", &row.stage_id, &row.tool_id);
        let matrix_row = vcf_matrix_rows.get(&key).ok_or_else(|| {
            anyhow!(
                "all-domain stage-tool table is missing VCF matrix coverage for `{}` / `{}`",
                row.stage_id,
                row.tool_id
            )
        })?;
        rows.push(AllDomainStageToolTableRow {
            domain: "vcf".to_string(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            corpus_id: matrix_row.corpus_id.clone(),
            asset_profile_id: matrix_row.asset_profile_id.clone(),
            adapter_id: matrix_row.adapter_id.clone(),
            parser_id: matrix_row.parser_id.clone(),
            benchmark_status: row.benchmark_status.clone(),
        });
    }

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_all_domain_stage_tool_table_contract(repo_root, &rows)?;
    Ok(rows)
}

fn ensure_all_domain_stage_tool_table_contract(
    repo_root: &Path,
    rows: &[AllDomainStageToolTableRow],
) -> Result<()> {
    let mut seen = BTreeSet::<BindingKey>::new();
    for row in rows {
        if !seen.insert(binding_key(&row.domain, &row.stage_id, &row.tool_id)) {
            return Err(anyhow!(
                "all-domain stage-tool table contains duplicate `{}` / `{}` / `{}`",
                row.domain,
                row.stage_id,
                row.tool_id
            ));
        }
        if row.corpus_id.trim().is_empty()
            || row.asset_profile_id.trim().is_empty()
            || row.adapter_id.trim().is_empty()
            || row.parser_id.trim().is_empty()
        {
            return Err(anyhow!(
                "all-domain stage-tool table row `{}` / `{}` / `{}` contains a blank required column",
                row.domain,
                row.stage_id,
                row.tool_id
            ));
        }
    }

    let expected_ready_keys = collect_fastq_command_adapter_coverage_rows(repo_root)?
        .2
        .into_iter()
        .filter(|row| row.benchmark_status == FastqBenchmarkStatus::BenchmarkReady)
        .map(|row| binding_key("fastq", &row.stage_id, &row.tool_id))
        .chain(
            collect_bam_command_adapter_coverage_rows(repo_root)?
                .2
                .into_iter()
                .filter(|row| row.benchmark_status == BamBenchmarkStatus::BenchmarkReady)
                .map(|row| binding_key("bam", &row.stage_id, &row.tool_id)),
        )
        .chain(
            collect_vcf_tool_serving_map_rows()?
                .into_iter()
                .filter(|row| row.benchmark_status == "benchmark_ready")
                .map(|row| binding_key("vcf", &row.stage_id, &row.tool_id)),
        )
        .collect::<BTreeSet<_>>();

    let observed_ready_keys = rows
        .iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .map(|row| binding_key(&row.domain, &row.stage_id, &row.tool_id))
        .collect::<BTreeSet<_>>();

    if expected_ready_keys != observed_ready_keys {
        let missing = expected_ready_keys
            .difference(&observed_ready_keys)
            .map(render_binding_key)
            .collect::<Vec<_>>();
        let extra = observed_ready_keys
            .difference(&expected_ready_keys)
            .map(render_binding_key)
            .collect::<Vec<_>>();
        return Err(anyhow!(
            "all-domain stage-tool table drifted from domain-ready bindings; missing={missing:?} extra={extra:?}"
        ));
    }

    let expected_vcf_row_count = vcf_matrix_rows_len()?;
    let actual_vcf_row_count = rows.iter().filter(|row| row.domain == "vcf").count();
    if actual_vcf_row_count != expected_vcf_row_count {
        return Err(anyhow!(
            "all-domain stage-tool table must include every governed VCF matrix row (expected {expected_vcf_row_count}, found {actual_vcf_row_count})"
        ));
    }

    Ok(())
}

fn fastq_bam_asset_profile_id(
    key: &BindingKey,
    asset_roles_by_binding: &BTreeMap<BindingKey, Vec<String>>,
) -> String {
    let Some(asset_roles) = asset_roles_by_binding.get(key) else {
        return "corpus_only".to_string();
    };
    let mut unique_roles = asset_roles.clone();
    unique_roles.sort();
    unique_roles.dedup();
    if unique_roles.is_empty() {
        "corpus_only".to_string()
    } else {
        unique_roles.join("+")
    }
}

fn load_committed_stage_tool_asset_rows(repo_root: &Path) -> Result<Vec<StageToolAssetRow>> {
    let config_path = repo_root.join(DEFAULT_STAGE_TOOL_ASSETS_PATH);
    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: StageToolAssetsConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    if config.schema_version != LOCAL_STAGE_TOOL_ASSETS_SCHEMA_VERSION {
        return Err(anyhow!(
            "committed stage-tool assets config drifted from governed schema `{}`",
            config.schema_version
        ));
    }
    Ok(config.rows)
}

fn normalized_adapter_surface_id(domain: &str, stage_id: &str) -> String {
    format!("{domain}.adapter.{}", stage_suffix(stage_id))
}

fn normalized_parser_surface_id(domain: &str, stage_id: &str) -> String {
    format!("{domain}.parser.{}", stage_suffix(stage_id))
}

fn stage_suffix(stage_id: &str) -> &str {
    stage_id.split_once('.').map_or(stage_id, |(_, suffix)| suffix)
}

fn fastq_benchmark_status_label(status: FastqBenchmarkStatus) -> &'static str {
    match status {
        FastqBenchmarkStatus::BenchmarkReady => "benchmark_ready",
        FastqBenchmarkStatus::NotBenchmarkReady => "not_benchmark_ready",
    }
}

fn bam_benchmark_status_label(status: BamBenchmarkStatus) -> &'static str {
    match status {
        BamBenchmarkStatus::BenchmarkReady => "benchmark_ready",
        BamBenchmarkStatus::NotBenchmarkReady => "not_benchmark_ready",
    }
}

fn binding_key(domain: &str, stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
    }
}

fn render_binding_key(key: &BindingKey) -> String {
    format!("{}:{}:{}", key.domain, key.stage_id, key.tool_id)
}

fn vcf_matrix_rows_len() -> Result<usize> {
    Ok(collect_vcf_benchmark_binding_rows()?.len())
}

fn render_all_domain_stage_tool_table_tsv(rows: &[AllDomainStageToolTableRow]) -> String {
    let mut rendered = String::from(
        "domain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tbenchmark_status\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.corpus_id),
            sanitize_tsv(&row.asset_profile_id),
            sanitize_tsv(&row.adapter_id),
            sanitize_tsv(&row.parser_id),
            sanitize_tsv(&row.benchmark_status),
        ));
    }
    rendered
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
        render_all_domain_stage_tool_table, ALL_DOMAIN_STAGE_TOOL_TABLE_SCHEMA_VERSION,
        DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn all_domain_stage_tool_table_reports_governed_ready_rows_once() {
        let root = repo_root();
        let report = render_all_domain_stage_tool_table(
            &root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH),
        )
        .expect("render all-domain stage-tool table");

        assert_eq!(report.schema_version, ALL_DOMAIN_STAGE_TOOL_TABLE_SCHEMA_VERSION);
        assert_eq!(report.row_count, report.rows.len());
        assert_eq!(report.benchmark_ready_row_count, report.benchmark_ready_unique_binding_count);
        assert!(report.rows.iter().any(|row| row.domain == "fastq"
            && row.stage_id == "fastq.validate_reads"
            && row.tool_id == "fastqc"
            && row.corpus_id == "corpus-01-mini"
            && row.asset_profile_id == "corpus_only"
            && row.adapter_id == "fastq.adapter.validate_reads"
            && row.parser_id == "fastq.parser.validate_reads"
            && row.benchmark_status == "benchmark_ready"));
        assert!(report.rows.iter().any(|row| row.domain == "bam"
            && row.stage_id == "bam.coverage"
            && row.tool_id == "mosdepth"
            && row.corpus_id == "corpus-01-mini"
            && row.asset_profile_id == "corpus_only"
            && row.adapter_id == "bam.adapter.coverage"
            && row.parser_id == "bam.parser.coverage"
            && row.benchmark_status == "benchmark_ready"));
        assert!(report.rows.iter().any(|row| row.domain == "vcf"
            && row.stage_id == "vcf.call"
            && row.tool_id == "bcftools"
            && row.corpus_id == "vcf_production_regression"
            && row.asset_profile_id == "bam_bundle"
            && row.adapter_id == "vcf.adapter.calling"
            && row.parser_id == "vcf.parser.call_summary"
            && row.benchmark_status == "benchmark_ready"));
    }
}
