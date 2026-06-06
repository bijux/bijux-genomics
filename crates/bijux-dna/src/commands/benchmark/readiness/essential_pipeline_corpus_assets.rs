use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_corpus_stage_compatibility::{
    validate_corpus_stage_compatibility_path, LocalCorpusStageCompatibilityEntryReport,
    DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH,
};
use crate::commands::benchmark::local_pipeline_dag::{
    validate_pipeline_dag_path, LocalPipelineDagValidationNodeReport,
    LocalPipelineDagValidationReport,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ESSENTIAL_PIPELINE_CORPUS_ASSETS_PATH: &str =
    "target/bench-readiness/essential-pipeline-corpus-assets.tsv";
const ESSENTIAL_PIPELINE_CORPUS_ASSETS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.essential_pipeline_corpus_assets.v1";
pub(crate) const ESSENTIAL_PIPELINE_IDS: &[&str] = &[
    "core-germline-fastq-bam-vcf",
    "adna-pseudohaploid-fastq-bam-vcf",
    "adna-gl-fastq-bam-vcf",
    "diploid-small-fastq-bam-vcf",
    "reference-panel-imputation",
    "popgen-structure-vcf",
    "relatedness-segments-vcf",
    "bam-genotyping-to-vcf-downstream",
    "edna-taxonomy-no-vcf",
    "amplicon-asv-otu-no-vcf",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct EssentialPipelineCorpusAssetsRow {
    pub(crate) pipeline_id: String,
    pub(crate) node_id: String,
    pub(crate) stage_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) input_paths: String,
    pub(crate) output_paths: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EssentialPipelineCorpusAssetsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) pipeline_count: usize,
    pub(crate) row_count: usize,
    pub(crate) resolved_row_count: usize,
    pub(crate) corpus_count: usize,
    pub(crate) asset_profile_count: usize,
    pub(crate) pipeline_row_counts: BTreeMap<String, usize>,
    pub(crate) domain_row_counts: BTreeMap<String, usize>,
    pub(crate) status_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<EssentialPipelineCorpusAssetsRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CorpusResolutionSource {
    FixtureBound,
    PipelineBound,
}

struct CorpusResolution {
    corpus_id: String,
    source: CorpusResolutionSource,
}

pub(crate) fn run_render_essential_pipeline_corpus_assets(
    args: &parse::BenchReadinessRenderEssentialPipelineCorpusAssetsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_essential_pipeline_corpus_assets(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_CORPUS_ASSETS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_essential_pipeline_corpus_assets(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<EssentialPipelineCorpusAssetsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_essential_pipeline_corpus_asset_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_essential_pipeline_corpus_assets_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let mut pipeline_row_counts = BTreeMap::<String, usize>::new();
    let mut domain_row_counts = BTreeMap::<String, usize>::new();
    let mut status_counts = BTreeMap::<String, usize>::new();
    let mut corpus_ids = BTreeSet::<String>::new();
    let mut asset_profile_ids = BTreeSet::<String>::new();
    for row in &rows {
        *pipeline_row_counts.entry(row.pipeline_id.clone()).or_default() += 1;
        *domain_row_counts.entry(stage_domain(row.stage_id.as_str()).to_string()).or_default() += 1;
        *status_counts.entry(row.status.clone()).or_default() += 1;
        corpus_ids.insert(row.corpus_id.clone());
        asset_profile_ids.insert(row.asset_profile_id.clone());
    }

    Ok(EssentialPipelineCorpusAssetsReport {
        schema_version: ESSENTIAL_PIPELINE_CORPUS_ASSETS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        pipeline_count: ESSENTIAL_PIPELINE_IDS.len(),
        row_count: rows.len(),
        resolved_row_count: rows.len(),
        corpus_count: corpus_ids.len(),
        asset_profile_count: asset_profile_ids.len(),
        pipeline_row_counts,
        domain_row_counts,
        status_counts,
        rows,
    })
}

pub(crate) fn collect_essential_pipeline_corpus_asset_rows(
    repo_root: &Path,
) -> Result<Vec<EssentialPipelineCorpusAssetsRow>> {
    let compatibility_report = validate_corpus_stage_compatibility_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
    )?;
    let compatibility_by_stage = compatibility_report
        .stages
        .into_iter()
        .map(|entry| (entry.stage_id.clone(), entry))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::new();
    for pipeline_id in ESSENTIAL_PIPELINE_IDS {
        let config_path =
            repo_root.join("configs/pipelines/local").join(format!("{pipeline_id}.toml"));
        let report_path =
            repo_root.join("target/local-ready/pipeline-dag").join(format!("{pipeline_id}.json"));
        let report = validate_pipeline_dag_path(repo_root, &config_path, &report_path)?;
        for node in &report.nodes {
            rows.push(build_row(&report, node, &compatibility_by_stage)?);
        }
    }

    rows.sort_by(|left, right| {
        left.pipeline_id
            .cmp(&right.pipeline_id)
            .then_with(|| left.node_id.cmp(&right.node_id))
            .then_with(|| left.stage_id.cmp(&right.stage_id))
    });
    ensure_essential_pipeline_corpus_asset_contract(&rows)?;
    Ok(rows)
}

fn build_row(
    pipeline: &LocalPipelineDagValidationReport,
    node: &LocalPipelineDagValidationNodeReport,
    compatibility_by_stage: &BTreeMap<String, LocalCorpusStageCompatibilityEntryReport>,
) -> Result<EssentialPipelineCorpusAssetsRow> {
    validate_symbol_set(pipeline.pipeline_id.as_str(), node)?;
    let corpus_resolution = resolve_corpus_binding(pipeline, node, compatibility_by_stage)?;
    let asset_profile_id = resolve_asset_profile_id(&node.external_inputs)?;
    let input_paths = render_input_paths(node);
    let output_paths = render_output_paths(node);
    let status = match corpus_resolution.source {
        CorpusResolutionSource::FixtureBound => "resolved_fixture_bound",
        CorpusResolutionSource::PipelineBound => "resolved_pipeline_bound",
    };

    Ok(EssentialPipelineCorpusAssetsRow {
        pipeline_id: pipeline.pipeline_id.clone(),
        node_id: node.node_id.clone(),
        stage_id: node.stage_id.clone(),
        corpus_id: corpus_resolution.corpus_id,
        asset_profile_id,
        input_paths,
        output_paths,
        status: status.to_string(),
    })
}

fn resolve_corpus_binding(
    pipeline: &LocalPipelineDagValidationReport,
    node: &LocalPipelineDagValidationNodeReport,
    compatibility_by_stage: &BTreeMap<String, LocalCorpusStageCompatibilityEntryReport>,
) -> Result<CorpusResolution> {
    if node.external_inputs.iter().any(|value| value.starts_with("corpus.")) {
        return Ok(CorpusResolution {
            corpus_id: pipeline.default_corpus_id.clone(),
            source: CorpusResolutionSource::PipelineBound,
        });
    }

    if node.stage_id.starts_with("vcf.") {
        return Ok(CorpusResolution {
            corpus_id: pipeline.default_corpus_id.clone(),
            source: CorpusResolutionSource::PipelineBound,
        });
    }

    let Some(entry) = compatibility_by_stage.get(&node.stage_id) else {
        return Err(anyhow!(
            "essential pipeline corpus/assets report is missing corpus-stage compatibility for `{}` in pipeline `{}`",
            node.stage_id,
            pipeline.pipeline_id
        ));
    };

    if let Some(corpus_family_id) = &entry.corpus_family_id {
        return Ok(CorpusResolution {
            corpus_id: corpus_family_id.clone(),
            source: CorpusResolutionSource::FixtureBound,
        });
    }

    Ok(CorpusResolution {
        corpus_id: pipeline.default_corpus_id.clone(),
        source: CorpusResolutionSource::PipelineBound,
    })
}

fn resolve_asset_profile_id(external_inputs: &[String]) -> Result<String> {
    if external_inputs.is_empty() {
        return Ok("upstream_only".to_string());
    }

    let mut non_corpus_contracts = external_inputs
        .iter()
        .filter(|value| !value.starts_with("corpus."))
        .cloned()
        .collect::<Vec<_>>();

    if non_corpus_contracts.is_empty() {
        return Ok("corpus_only".to_string());
    }

    for contract_id in &non_corpus_contracts {
        ensure_not_path_like(contract_id, "asset binding")?;
    }

    non_corpus_contracts.sort();
    non_corpus_contracts.dedup();
    Ok(non_corpus_contracts.join("+"))
}

fn render_input_paths(node: &LocalPipelineDagValidationNodeReport) -> String {
    node.external_inputs
        .iter()
        .map(|value| format!("external:{value}"))
        .chain(node.upstream_inputs.iter().map(|value| format!("upstream:{value}")))
        .collect::<Vec<_>>()
        .join(",")
}

fn render_output_paths(node: &LocalPipelineDagValidationNodeReport) -> String {
    node.outputs.iter().map(|value| format!("output:{value}")).collect::<Vec<_>>().join(",")
}

fn validate_symbol_set(
    pipeline_id: &str,
    node: &LocalPipelineDagValidationNodeReport,
) -> Result<()> {
    ensure_not_path_like(node.node_id.as_str(), "node_id")?;
    ensure_not_path_like(node.stage_id.as_str(), "stage_id")?;
    for symbol in &node.external_inputs {
        ensure_not_path_like(symbol, "external input")?;
    }
    for symbol in &node.upstream_inputs {
        ensure_not_path_like(symbol, "upstream input")?;
    }
    for symbol in &node.outputs {
        ensure_not_path_like(symbol, "output")?;
    }
    if node.outputs.is_empty() {
        return Err(anyhow!(
            "essential pipeline corpus/assets report found node `{}` in pipeline `{pipeline_id}` without outputs",
            node.node_id
        ));
    }
    Ok(())
}

fn ensure_not_path_like(value: &str, label: &str) -> Result<()> {
    if value.contains('/') || value.contains('\\') {
        return Err(anyhow!(
            "essential pipeline corpus/assets report does not allow path-like {label} `{value}`"
        ));
    }
    if value.starts_with('~') || value.starts_with('.') {
        return Err(anyhow!(
            "essential pipeline corpus/assets report does not allow implicit {label} `{value}`"
        ));
    }
    Ok(())
}

fn ensure_essential_pipeline_corpus_asset_contract(
    rows: &[EssentialPipelineCorpusAssetsRow],
) -> Result<()> {
    let unique_nodes = rows
        .iter()
        .map(|row| format!("{}:{}", row.pipeline_id, row.node_id))
        .collect::<BTreeSet<_>>();
    if unique_nodes.len() != rows.len() {
        return Err(anyhow!(
            "essential pipeline corpus/assets report must keep exactly one row per pipeline node"
        ));
    }
    if rows.iter().map(|row| row.pipeline_id.as_str()).collect::<BTreeSet<_>>().len()
        != ESSENTIAL_PIPELINE_IDS.len()
    {
        return Err(anyhow!(
            "essential pipeline corpus/assets report must cover all {} governed essential pipelines",
            ESSENTIAL_PIPELINE_IDS.len()
        ));
    }
    for row in rows {
        if row.corpus_id.trim().is_empty()
            || row.asset_profile_id.trim().is_empty()
            || row.input_paths.trim().is_empty()
            || row.output_paths.trim().is_empty()
            || row.status.trim().is_empty()
        {
            return Err(anyhow!(
                "essential pipeline corpus/assets row `{}` / `{}` is missing required columns",
                row.pipeline_id,
                row.node_id
            ));
        }
    }
    Ok(())
}

fn render_essential_pipeline_corpus_assets_tsv(
    rows: &[EssentialPipelineCorpusAssetsRow],
) -> String {
    let mut rendered = String::from(
        "pipeline_id\tnode_id\tstage_id\tcorpus_id\tasset_profile_id\tinput_paths\toutput_paths\tstatus\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.pipeline_id),
            sanitize_tsv(&row.node_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.corpus_id),
            sanitize_tsv(&row.asset_profile_id),
            sanitize_tsv(&row.input_paths),
            sanitize_tsv(&row.output_paths),
            sanitize_tsv(&row.status),
        ));
    }
    rendered
}

fn stage_domain(stage_id: &str) -> &'static str {
    if stage_id.starts_with("fastq.") {
        "fastq"
    } else if stage_id.starts_with("bam.") {
        "bam"
    } else if stage_id.starts_with("vcf.") {
        "vcf"
    } else {
        "unknown"
    }
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn sanitize_tsv(value: &str) -> String {
    value.replace('\t', " ").replace('\n', " ").replace('\r', " ")
}
