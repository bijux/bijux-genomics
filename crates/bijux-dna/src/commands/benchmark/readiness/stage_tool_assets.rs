use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_stage_contract::StagePlanV1;
use serde::{Deserialize, Serialize};

use super::bam_command_adapter_coverage::{
    collect_bam_command_adapter_coverage_rows, BamBenchmarkStatus,
};
use super::fastq_command_adapter_coverage::{
    collect_fastq_command_adapter_coverage_rows, FastqBenchmarkStatus,
};
use crate::commands::benchmark::local_stage_commands::collect_local_stage_plan_bundles;
use crate::commands::benchmark::local_stage_inventory::BenchLocalDomain;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_STAGE_TOOL_ASSETS_PATH: &str =
    "configs/bench/local/stage-tool-assets.toml";
pub(crate) const LOCAL_STAGE_TOOL_ASSETS_SCHEMA_VERSION: &str =
    "bijux.bench.local_stage_tool_assets.v1";
const STAGE_TOOL_ASSETS_REPORT_SCHEMA_VERSION: &str = "bijux.bench.readiness.stage_tool_assets.v1";
const STAGE_TOOL_ASSETS_SCOPE: &str = "governed_benchmark_command_assets";
const STAGE_TOOL_ASSETS_DECLARATION_ORIGIN: &str =
    "canonical_local_stage_plan_inputs_and_governed_stage_params";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageToolAssetsConfig {
    pub(crate) schema_version: String,
    pub(crate) classification_scope: String,
    pub(crate) rows: Vec<StageToolAssetRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageToolAssetRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) asset_role: String,
    pub(crate) asset_id: String,
    pub(crate) asset_path: String,
    pub(crate) declaration_origin: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct StageToolAssetsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) config_path: String,
    pub(crate) classification_scope: &'static str,
    pub(crate) row_count: usize,
    pub(crate) declared_stage_tool_row_count: usize,
    pub(crate) asset_id_row_count: usize,
    pub(crate) unique_asset_id_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) asset_role_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<StageToolAssetRow>,
}

pub(crate) fn run_render_stage_tool_assets(
    args: &parse::BenchReadinessRenderStageToolAssetsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_stage_tool_assets(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_STAGE_TOOL_ASSETS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.config_path);
    }
    Ok(())
}

pub(crate) fn render_stage_tool_assets(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<StageToolAssetsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_stage_tool_asset_rows(repo_root)?;

    let config = StageToolAssetsConfig {
        schema_version: LOCAL_STAGE_TOOL_ASSETS_SCHEMA_VERSION.to_string(),
        classification_scope: STAGE_TOOL_ASSETS_SCOPE.to_string(),
        rows: rows.clone(),
    };
    let rendered = render_stage_tool_assets_toml(&config)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered.as_bytes())?;

    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut asset_role_counts = BTreeMap::<String, usize>::new();
    let mut declared_stage_tool_rows = BTreeSet::<(String, String)>::new();
    let mut unique_asset_ids = BTreeSet::<String>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *asset_role_counts.entry(row.asset_role.clone()).or_default() += 1;
        declared_stage_tool_rows.insert((row.stage_id.clone(), row.tool_id.clone()));
        unique_asset_ids.insert(row.asset_id.clone());
    }

    Ok(StageToolAssetsReport {
        schema_version: STAGE_TOOL_ASSETS_REPORT_SCHEMA_VERSION,
        config_path: path_relative_to_repo(repo_root, &output_path),
        classification_scope: STAGE_TOOL_ASSETS_SCOPE,
        row_count: rows.len(),
        declared_stage_tool_row_count: declared_stage_tool_rows.len(),
        asset_id_row_count: rows.iter().filter(|row| !row.asset_id.trim().is_empty()).count(),
        unique_asset_id_count: unique_asset_ids.len(),
        domain_counts,
        asset_role_counts,
        rows,
    })
}

fn collect_stage_tool_asset_rows(repo_root: &Path) -> Result<Vec<StageToolAssetRow>> {
    let fastq_base_plans = canonical_stage_plan_map(repo_root, BenchLocalDomain::Fastq)?;
    let bam_base_plans = canonical_stage_plan_map(repo_root, BenchLocalDomain::Bam)?;
    let (_, _, fastq_rows) = collect_fastq_command_adapter_coverage_rows(repo_root)?;
    let (_, _, bam_rows) = collect_bam_command_adapter_coverage_rows(repo_root)?;

    let mut rows = Vec::new();
    for row in fastq_rows
        .into_iter()
        .filter(fastq_asset_declaration_required)
    {
        let Some(plan) = fastq_base_plans.get(&row.stage_id) else {
            continue;
        };
        if row.stage_id == "fastq.index_reference" && row.tool_id != plan.tool_id.as_str() {
            continue;
        }
        rows.extend(render_fastq_stage_tool_asset_rows(&row.stage_id, &row.tool_id, plan)?);
    }

    for row in bam_rows
        .into_iter()
        .filter(|row| row.benchmark_status == BamBenchmarkStatus::BenchmarkReady)
    {
        let Some(plan) = bam_base_plans.get(&row.stage_id) else {
            continue;
        };
        rows.extend(render_bam_stage_tool_asset_rows(&row.stage_id, &row.tool_id, plan)?);
    }

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.asset_role.cmp(&right.asset_role))
            .then_with(|| left.asset_id.cmp(&right.asset_id))
    });
    ensure_unique_stage_tool_asset_rows(&rows)?;
    ensure_taxonomy_database_asset_coverage(&rows)?;
    ensure_bam_adna_asset_coverage(&rows)?;
    Ok(rows)
}

fn fastq_asset_declaration_required(
    row: &super::fastq_command_adapter_coverage::FastqCommandAdapterCoverageRow,
) -> bool {
    row.benchmark_status == FastqBenchmarkStatus::BenchmarkReady
        || row.stage_id == "fastq.index_reference"
}

fn canonical_stage_plan_map(
    repo_root: &Path,
    domain: BenchLocalDomain,
) -> Result<BTreeMap<String, StagePlanV1>> {
    collect_local_stage_plan_bundles(repo_root, Some(domain))?
        .into_iter()
        .map(|bundle| {
            let plan = bundle.plans.into_iter().next().ok_or_else(|| {
                anyhow!(
                    "local stage bundle `{}` did not include a canonical governed plan",
                    bundle.stage_id
                )
            })?;
            Ok((bundle.stage_id, plan))
        })
        .collect()
}

fn render_fastq_stage_tool_asset_rows(
    stage_id: &str,
    tool_id: &str,
    plan: &StagePlanV1,
) -> Result<Vec<StageToolAssetRow>> {
    match stage_id {
        "fastq.screen_taxonomy" => {
            let path = find_input_path(plan, &["taxonomy_database_root"])?;
            Ok(vec![
                asset_row(
                    "fastq",
                    stage_id,
                    tool_id,
                    "taxonomy_database_root",
                    json_string(plan.effective_params.get("database_catalog_id"))
                        .unwrap_or_else(|| asset_id_from_path(&path)),
                    path.clone(),
                ),
                asset_row(
                    "fastq",
                    stage_id,
                    tool_id,
                    "database_artifact_id",
                    json_string(plan.effective_params.get("database_artifact_id"))
                        .ok_or_else(|| {
                            anyhow!(
                                "canonical local stage plan `{stage_id}` / `{tool_id}` is missing governed `database_artifact_id`"
                            )
                        })?,
                    path,
                ),
            ])
        }
        "fastq.deplete_host" => {
            let path = find_input_path(plan, &["reference_index"])?;
            Ok(vec![
                asset_row(
                    "fastq",
                    stage_id,
                    tool_id,
                    "reference_catalog_id",
                    json_string(plan.effective_params.get("reference_catalog_id"))
                        .unwrap_or_else(|| asset_id_from_path(&path)),
                    path.clone(),
                ),
                asset_row(
                    "fastq",
                    stage_id,
                    tool_id,
                    "reference_index_artifact_id",
                    json_string(plan.effective_params.get("reference_index_artifact_id"))
                        .ok_or_else(|| {
                            anyhow!(
                                "canonical local stage plan `{stage_id}` / `{tool_id}` is missing governed `reference_index_artifact_id`"
                            )
                        })?,
                    path,
                ),
            ])
        }
        "fastq.deplete_reference_contaminants" => {
            let path = find_input_path(plan, &["reference_index"])?;
            Ok(vec![
                asset_row(
                    "fastq",
                    stage_id,
                    tool_id,
                    "reference_catalog_id",
                    json_string(plan.effective_params.get("reference_catalog_id"))
                        .unwrap_or_else(|| asset_id_from_path(&path)),
                    path.clone(),
                ),
                asset_row(
                    "fastq",
                    stage_id,
                    tool_id,
                    "reference_index_artifact_id",
                    json_string(plan.effective_params.get("index_artifact")).ok_or_else(|| {
                        anyhow!(
                            "canonical local stage plan `{stage_id}` / `{tool_id}` is missing governed `index_artifact`"
                        )
                    })?,
                    path,
                ),
            ])
        }
        "fastq.deplete_rrna" => {
            let path = find_input_path(plan, &["rrna_reference"])?;
            Ok(vec![
                asset_row(
                    "fastq",
                    stage_id,
                    tool_id,
                    "rrna_reference",
                    json_string(plan.effective_params.get("database_artifact_id"))
                        .unwrap_or_else(|| asset_id_from_path(&path)),
                    path.clone(),
                ),
                asset_row(
                    "fastq",
                    stage_id,
                    tool_id,
                    "database_artifact_id",
                    json_string(plan.effective_params.get("database_artifact_id"))
                        .ok_or_else(|| {
                            anyhow!(
                                "canonical local stage plan `{stage_id}` / `{tool_id}` is missing governed `database_artifact_id`"
                            )
                        })?,
                    path,
                ),
            ])
        }
        "fastq.index_reference" => {
            let reference_path = find_input_path(plan, &["reference_fasta"])?;
            let index_path = find_output_path(plan, &["reference_index"])?;
            Ok(vec![
                asset_row(
                    "fastq",
                    stage_id,
                    tool_id,
                    "reference_fasta",
                    asset_id_from_path(&reference_path),
                    reference_path,
                ),
                asset_row(
                    "fastq",
                    stage_id,
                    tool_id,
                    "reference_index_output",
                    "reference_index".to_string(),
                    index_path,
                ),
            ])
        }
        _ => Ok(Vec::new()),
    }
}

fn ensure_taxonomy_database_asset_coverage(rows: &[StageToolAssetRow]) -> Result<()> {
    let taxonomy_rows = rows
        .iter()
        .filter(|row| {
            row.domain == "fastq"
                && row.stage_id == "fastq.screen_taxonomy"
                && row.asset_role == "taxonomy_database_root"
        })
        .collect::<Vec<_>>();
    let expected_tool_ids = ["centrifuge", "kaiju", "kraken2", "krakenuniq"];
    if taxonomy_rows.len() != expected_tool_ids.len() {
        return Err(anyhow!(
            "FASTQ taxonomy asset coverage expected {} rows but found {}",
            expected_tool_ids.len(),
            taxonomy_rows.len()
        ));
    }
    for tool_id in expected_tool_ids {
        let row = taxonomy_rows
            .iter()
            .find(|row| row.tool_id == tool_id)
            .ok_or_else(|| anyhow!("FASTQ taxonomy asset coverage is missing `{tool_id}`"))?;
        if row.asset_id != "taxonomy_reference"
            || row.asset_path != "assets/reference/taxonomy/references/mock_community_taxonomy"
        {
            return Err(anyhow!(
                "FASTQ taxonomy asset row `{}` must stay bound to the governed taxonomy reference root",
                tool_id
            ));
        }
    }
    for tool_id in expected_tool_ids {
        ensure_stage_tool_asset_row(
            rows,
            "fastq.screen_taxonomy",
            tool_id,
            "database_artifact_id",
            "taxonomy_db",
            "assets/reference/taxonomy/references/mock_community_taxonomy",
            "FASTQ taxonomy",
        )?;
    }
    ensure_stage_tool_asset_row(
        rows,
        "fastq.deplete_rrna",
        "sortmerna",
        "rrna_reference",
        "sortmerna_common_rrna_reference",
        "assets/reference/rrna/references/sortmerna_common_rrna_reference.fasta",
        "FASTQ rRNA depletion",
    )?;
    ensure_stage_tool_asset_row(
        rows,
        "fastq.deplete_rrna",
        "sortmerna",
        "database_artifact_id",
        "sortmerna_common_rrna_reference",
        "assets/reference/rrna/references/sortmerna_common_rrna_reference.fasta",
        "FASTQ rRNA depletion",
    )?;
    ensure_stage_tool_asset_row(
        rows,
        "fastq.deplete_host",
        "bowtie2",
        "reference_catalog_id",
        "host_reference",
        "assets/reference/host/references/toy_host_reference",
        "FASTQ host depletion",
    )?;
    ensure_stage_tool_asset_row(
        rows,
        "fastq.deplete_host",
        "bowtie2",
        "reference_index_artifact_id",
        "reference_index",
        "assets/reference/host/references/toy_host_reference",
        "FASTQ host depletion",
    )?;
    ensure_stage_tool_asset_row(
        rows,
        "fastq.deplete_reference_contaminants",
        "bowtie2",
        "reference_catalog_id",
        "contaminant_reference",
        "assets/reference/contaminants/references/toy_contaminant_reference",
        "FASTQ contaminant depletion",
    )?;
    ensure_stage_tool_asset_row(
        rows,
        "fastq.deplete_reference_contaminants",
        "bowtie2",
        "reference_index_artifact_id",
        "reference_index",
        "assets/reference/contaminants/references/toy_contaminant_reference",
        "FASTQ contaminant depletion",
    )?;
    ensure_stage_tool_asset_row(
        rows,
        "fastq.index_reference",
        "bowtie2_build",
        "reference_fasta",
        "phix174",
        "assets/reference/contaminants/references/phix174.fasta",
        "FASTQ reference indexing",
    )?;
    ensure_stage_tool_asset_row(
        rows,
        "fastq.index_reference",
        "bowtie2_build",
        "reference_index_output",
        "reference_index",
        "target/local-ready/fastq.index_reference/reference_index/bowtie2/reference",
        "FASTQ reference indexing",
    )?;
    Ok(())
}

fn ensure_bam_adna_asset_coverage(rows: &[StageToolAssetRow]) -> Result<()> {
    for tool_id in ["contammix", "schmutzi", "verifybamid2"] {
        ensure_stage_tool_asset_row(
            rows,
            "bam.contamination",
            tool_id,
            "reference_fasta",
            "adna_bam_reference",
            "tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta",
            "BAM contamination",
        )?;
        ensure_stage_tool_asset_row(
            rows,
            "bam.contamination",
            tool_id,
            "reference_panel",
            "adna_contamination_panel",
            "tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_contamination_panel.dat",
            "BAM contamination",
        )?;
    }

    for tool_id in ["angsd", "rxy", "yleaf"] {
        ensure_stage_tool_asset_row(
            rows,
            "bam.sex",
            tool_id,
            "reference_fasta",
            "adna_bam_reference",
            "tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta",
            "BAM sex",
        )?;
    }

    ensure_stage_tool_asset_row(
        rows,
        "bam.haplogroups",
        "yleaf",
        "reference_fasta",
        "adna_bam_reference",
        "tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta",
        "BAM haplogroups",
    )?;
    ensure_stage_tool_asset_row(
        rows,
        "bam.haplogroups",
        "yleaf",
        "reference_panel",
        "adna-y-hg38-mini",
        "tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_y_haplogroup_panel.tsv",
        "BAM haplogroups",
    )?;

    Ok(())
}

fn ensure_stage_tool_asset_row(
    rows: &[StageToolAssetRow],
    stage_id: &str,
    tool_id: &str,
    asset_role: &str,
    asset_id: &str,
    asset_path: &str,
    label: &str,
) -> Result<()> {
    let row = rows
        .iter()
        .find(|row| {
            row.stage_id == stage_id && row.tool_id == tool_id && row.asset_role == asset_role
        })
        .ok_or_else(|| anyhow!("{label} asset coverage is missing `{stage_id}` / `{tool_id}` / `{asset_role}`"))?;
    if row.asset_id != asset_id || row.asset_path != asset_path {
        return Err(anyhow!(
            "{label} asset row `{stage_id}` / `{tool_id}` / `{asset_role}` must stay bound to `{asset_id}` at `{asset_path}`"
        ));
    }
    Ok(())
}

fn render_bam_stage_tool_asset_rows(
    stage_id: &str,
    tool_id: &str,
    plan: &StagePlanV1,
) -> Result<Vec<StageToolAssetRow>> {
    match stage_id {
        "bam.contamination" => {
            let reference_path = find_input_path(plan, &["reference", "reference_fasta"])?;
            let panel_path = find_input_path(plan, &["reference_panel"])?;
            Ok(vec![
                asset_row(
                    "bam",
                    stage_id,
                    tool_id,
                    "reference_fasta",
                    asset_id_from_path(&reference_path),
                    reference_path,
                ),
                asset_row(
                    "bam",
                    stage_id,
                    tool_id,
                    "reference_panel",
                    asset_id_from_path(&panel_path),
                    panel_path,
                ),
            ])
        }
        "bam.haplogroups" => {
            let reference_path = find_input_path(plan, &["reference", "reference_fasta"])?;
            let panel_path = find_input_path(plan, &["reference_panel"])?;
            Ok(vec![
                asset_row(
                    "bam",
                    stage_id,
                    tool_id,
                    "reference_fasta",
                    asset_id_from_path(&reference_path),
                    reference_path,
                ),
                asset_row(
                    "bam",
                    stage_id,
                    tool_id,
                    "reference_panel",
                    json_string(plan.params.get("reference_panel_id"))
                        .unwrap_or_else(|| asset_id_from_path(&panel_path)),
                    panel_path,
                ),
            ])
        }
        "bam.sex" => {
            let reference_path = find_input_path(plan, &["reference", "reference_fasta"])?;
            Ok(vec![asset_row(
                "bam",
                stage_id,
                tool_id,
                "reference_fasta",
                asset_id_from_path(&reference_path),
                reference_path,
            )])
        }
        "bam.genotyping" => {
            let reference_path = find_input_path(plan, &["reference", "reference_fasta"])?;
            let sites_path = find_input_path(plan, &["sites", "sites_vcf"])?;
            let regions_path = find_input_path(plan, &["regions"])?;
            Ok(vec![
                asset_row(
                    "bam",
                    stage_id,
                    tool_id,
                    "reference_fasta",
                    asset_id_from_path(&reference_path),
                    reference_path,
                ),
                asset_row(
                    "bam",
                    stage_id,
                    tool_id,
                    "sites_vcf",
                    asset_id_from_path(&sites_path),
                    sites_path,
                ),
                asset_row(
                    "bam",
                    stage_id,
                    tool_id,
                    "regions",
                    asset_id_from_path(&regions_path),
                    regions_path,
                ),
            ])
        }
        "bam.recalibration" => {
            let reference_path = find_input_path(plan, &["reference", "reference_fasta"])?;
            let mut rows = vec![asset_row(
                "bam",
                stage_id,
                tool_id,
                "reference_fasta",
                asset_id_from_path(&reference_path),
                reference_path,
            )];
            for known_sites in json_string_list(plan.params.get("known_sites"))? {
                let known_sites_path = PathBuf::from(known_sites);
                rows.push(asset_row(
                    "bam",
                    stage_id,
                    tool_id,
                    "known_sites",
                    asset_id_from_path(&known_sites_path),
                    known_sites_path,
                ));
            }
            Ok(rows)
        }
        _ => Ok(Vec::new()),
    }
}

fn asset_row(
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    asset_role: &str,
    asset_id: String,
    asset_path: PathBuf,
) -> StageToolAssetRow {
    StageToolAssetRow {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        asset_role: asset_role.to_string(),
        asset_id,
        asset_path: asset_path.display().to_string(),
        declaration_origin: STAGE_TOOL_ASSETS_DECLARATION_ORIGIN.to_string(),
    }
}

fn find_input_path(plan: &StagePlanV1, names: &[&str]) -> Result<PathBuf> {
    for name in names {
        if let Some(artifact) =
            plan.io.inputs.iter().find(|artifact| artifact.name.as_str() == *name)
        {
            return Ok(artifact.path.clone());
        }
    }
    Err(anyhow!(
        "canonical local stage plan `{}` / `{}` is missing required asset input from {:?}",
        plan.stage_id.as_str(),
        plan.tool_id.as_str(),
        names
    ))
}

fn find_output_path(plan: &StagePlanV1, names: &[&str]) -> Result<PathBuf> {
    for name in names {
        if let Some(artifact) =
            plan.io.outputs.iter().find(|artifact| artifact.name.as_str() == *name)
        {
            return Ok(artifact.path.clone());
        }
    }
    Err(anyhow!(
        "canonical local stage plan `{}` / `{}` is missing required asset output from {:?}",
        plan.stage_id.as_str(),
        plan.tool_id.as_str(),
        names
    ))
}

fn json_string(value: Option<&serde_json::Value>) -> Option<String> {
    value
        .and_then(serde_json::Value::as_str)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn json_string_list(value: Option<&serde_json::Value>) -> Result<Vec<String>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let array = value
        .as_array()
        .ok_or_else(|| anyhow!("expected JSON array of strings for governed asset list"))?;
    array
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| anyhow!("expected non-empty string in governed asset list"))
        })
        .collect()
}

fn asset_id_from_path(path: &Path) -> String {
    let file_name = path
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    for suffix in [
        ".vcf.gz",
        ".fastq.gz",
        ".fq.gz",
        ".fasta",
        ".fastq",
        ".sam",
        ".bam",
        ".bai",
        ".vcf",
        ".tsv",
        ".txt",
        ".bed",
        ".dat",
        ".fa",
        ".fna",
        ".gz",
    ] {
        if let Some(stripped) = file_name.strip_suffix(suffix) {
            return stripped.to_string();
        }
    }
    file_name
}

fn ensure_unique_stage_tool_asset_rows(rows: &[StageToolAssetRow]) -> Result<()> {
    let mut seen = BTreeSet::<(String, String, String, String)>::new();
    for row in rows {
        let identity = (
            row.stage_id.clone(),
            row.tool_id.clone(),
            row.asset_role.clone(),
            row.asset_id.clone(),
        );
        if !seen.insert(identity.clone()) {
            return Err(anyhow!(
                "stage-tool asset rows repeat benchmark-ready asset binding `{}` / `{}` / `{}` / `{}`",
                identity.0,
                identity.1,
                identity.2,
                identity.3
            ));
        }
    }
    Ok(())
}

fn render_stage_tool_assets_toml(config: &StageToolAssetsConfig) -> Result<String> {
    let body = toml::to_string_pretty(config).context("serialize stage-tool assets config")?;
    Ok(format!(
        "# schema_version = 1\n\
         # owner = bijux-dna-bench\n\
         # purpose = Governed asset declarations for local benchmark FASTQ and BAM stage-tool commands.\n\
         # authority = bijux-dna-bench\n\
         # stability = evolving\n\n{body}"
    ))
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn render_stage_tool_assets_reports_governed_asset_rows() {
        use super::{render_stage_tool_assets, DEFAULT_STAGE_TOOL_ASSETS_PATH};

        let root = repo_root();
        let report = render_stage_tool_assets(&root, PathBuf::from(DEFAULT_STAGE_TOOL_ASSETS_PATH))
            .expect("render stage-tool assets");

        assert_eq!(report.schema_version, "bijux.bench.readiness.stage_tool_assets.v1");
        assert_eq!(report.config_path, "configs/bench/local/stage-tool-assets.toml");
        assert_eq!(report.classification_scope, "governed_benchmark_command_assets");
        assert_eq!(report.row_count, 32);
        assert_eq!(report.declared_stage_tool_row_count, 17);
        assert_eq!(report.asset_id_row_count, 32);
        assert_eq!(report.unique_asset_id_count, 14);
        assert_eq!(report.domain_counts.get("fastq"), Some(&16));
        assert_eq!(report.domain_counts.get("bam"), Some(&16));
        assert_eq!(report.asset_role_counts.get("taxonomy_database_root"), Some(&4));
        assert_eq!(report.asset_role_counts.get("database_artifact_id"), Some(&5));
        assert_eq!(report.asset_role_counts.get("reference_catalog_id"), Some(&2));
        assert_eq!(report.asset_role_counts.get("reference_index_artifact_id"), Some(&2));
        assert_eq!(report.asset_role_counts.get("rrna_reference"), Some(&1));
        assert_eq!(report.asset_role_counts.get("reference_fasta"), Some(&10));
        assert_eq!(report.asset_role_counts.get("reference_index_output"), Some(&1));
        assert_eq!(report.asset_role_counts.get("reference_panel"), Some(&4));
        assert_eq!(report.asset_role_counts.get("sites_vcf"), Some(&1));
        assert_eq!(report.asset_role_counts.get("regions"), Some(&1));
        assert_eq!(report.asset_role_counts.get("known_sites"), Some(&1));
        let taxonomy_rows = report
            .rows
            .iter()
            .filter(|row| {
                row.stage_id == "fastq.screen_taxonomy"
                    && row.asset_role == "taxonomy_database_root"
            })
            .collect::<Vec<_>>();
        assert_eq!(taxonomy_rows.len(), 4);
        for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
            assert!(taxonomy_rows.iter().any(|row| {
                row.tool_id == tool_id
                    && row.asset_id == "taxonomy_reference"
                    && row.asset_path
                        == "assets/reference/taxonomy/references/mock_community_taxonomy"
            }));
            assert!(report.rows.iter().any(|row| {
                row.stage_id == "fastq.screen_taxonomy"
                    && row.tool_id == tool_id
                    && row.asset_role == "database_artifact_id"
                    && row.asset_id == "taxonomy_db"
                    && row.asset_path
                        == "assets/reference/taxonomy/references/mock_community_taxonomy"
            }));
        }
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.deplete_host"
                && row.tool_id == "bowtie2"
                && row.asset_role == "reference_catalog_id"
                && row.asset_id == "host_reference"
                && row.asset_path == "assets/reference/host/references/toy_host_reference"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.deplete_host"
                && row.tool_id == "bowtie2"
                && row.asset_role == "reference_index_artifact_id"
                && row.asset_id == "reference_index"
                && row.asset_path == "assets/reference/host/references/toy_host_reference"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.deplete_reference_contaminants"
                && row.tool_id == "bowtie2"
                && row.asset_role == "reference_catalog_id"
                && row.asset_id == "contaminant_reference"
                && row.asset_path
                    == "assets/reference/contaminants/references/toy_contaminant_reference"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.deplete_reference_contaminants"
                && row.tool_id == "bowtie2"
                && row.asset_role == "reference_index_artifact_id"
                && row.asset_id == "reference_index"
                && row.asset_path
                    == "assets/reference/contaminants/references/toy_contaminant_reference"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.deplete_rrna"
                && row.tool_id == "sortmerna"
                && row.asset_role == "rrna_reference"
                && row.asset_id == "sortmerna_common_rrna_reference"
                && row.asset_path
                    == "assets/reference/rrna/references/sortmerna_common_rrna_reference.fasta"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.deplete_rrna"
                && row.tool_id == "sortmerna"
                && row.asset_role == "database_artifact_id"
                && row.asset_id == "sortmerna_common_rrna_reference"
                && row.asset_path
                    == "assets/reference/rrna/references/sortmerna_common_rrna_reference.fasta"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.index_reference"
                && row.tool_id == "bowtie2_build"
                && row.asset_role == "reference_fasta"
                && row.asset_id == "phix174"
                && row.asset_path == "assets/reference/contaminants/references/phix174.fasta"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.index_reference"
                && row.tool_id == "bowtie2_build"
                && row.asset_role == "reference_index_output"
                && row.asset_id == "reference_index"
                && row.asset_path
                    == "target/local-ready/fastq.index_reference/reference_index/bowtie2/reference"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.contamination"
                && row.tool_id == "verifybamid2"
                && row.asset_role == "reference_panel"
                && row.asset_id == "adna_contamination_panel"
                && row.asset_path
                    == "tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_contamination_panel.dat"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.sex"
                && row.tool_id == "rxy"
                && row.asset_role == "reference_fasta"
                && row.asset_id == "adna_bam_reference"
                && row.asset_path
                    == "tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.haplogroups"
                && row.tool_id == "yleaf"
                && row.asset_role == "reference_panel"
                && row.asset_id == "adna-y-hg38-mini"
                && row.asset_path
                    == "tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_y_haplogroup_panel.tsv"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.genotyping"
                && row.tool_id == "angsd"
                && row.asset_role == "sites_vcf"
                && row.asset_id == "human_like_genotyping_candidate_sites"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.recalibration"
                && row.tool_id == "gatk"
                && row.asset_role == "known_sites"
                && row.asset_id == "human_like_recalibration_known_sites"
        }));
    }
}
