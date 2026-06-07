use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::taxonomy::VcfDomainStage;
use bijux_dna_stages_vcf::stage_specs::{
    vcf_domain_stage_adapter_id, vcf_domain_stage_expected_output_ids, vcf_domain_stage_parser_id,
};
use serde::Serialize;

use crate::commands::benchmark::local_vcf_panel_workflow_smoke_support::materialize_governed_vcf_panel_assets;
use crate::commands::benchmark::local_vcf_stage_catalog::{
    build_vcf_stage_catalog_rows, VcfStageCatalogRow,
};
use crate::commands::benchmark::local_vcf_stage_matrix::{
    build_vcf_stage_matrix_rows, VcfStageMatrixRow,
};
use crate::commands::benchmark::readiness::vcf_readiness_inputs::{
    load_governed_vcf_fixture_inputs, materialize_indexed_vcf_input,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH: &str =
    "benchmarks/readiness/adapters/shapeit5.vcf.json";
pub(crate) const DEFAULT_VCF_EAGLE_ADAPTER_PATH: &str =
    "benchmarks/readiness/adapters/eagle.vcf.json";
pub(crate) const DEFAULT_VCF_BEAGLE_ADAPTER_PATH: &str =
    "benchmarks/readiness/adapters/beagle.vcf.json";
const VCF_SHAPEIT5_ADAPTER_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_shapeit5_adapter.v1";
const VCF_EAGLE_ADAPTER_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_eagle_adapter.v1";
const VCF_BEAGLE_ADAPTER_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_beagle_adapter.v1";
const GOVERNED_COHORT_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfPhasingFamilyAdapterArtifact {
    pub(crate) artifact_id: String,
    pub(crate) role: String,
    pub(crate) path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfPhasingFamilyAdapterCommandStep {
    pub(crate) step_id: String,
    pub(crate) step_kind: String,
    pub(crate) argv: Vec<String>,
    pub(crate) declared_output_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfPhasingFamilyAdapterRow {
    pub(crate) tool_id: String,
    pub(crate) tool_status: String,
    pub(crate) stage_id: String,
    pub(crate) stage_status: String,
    pub(crate) benchmark_status: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) command_contract_source: String,
    pub(crate) output_root: String,
    pub(crate) output_prefix: String,
    pub(crate) panel_id: String,
    pub(crate) map_id: String,
    pub(crate) input_vcf_path: String,
    pub(crate) panel_vcf_path: String,
    pub(crate) genetic_map_path: String,
    pub(crate) phased_vcf_path: String,
    pub(crate) phased_vcf_tbi_path: String,
    pub(crate) phase_block_stats_path: String,
    pub(crate) switch_error_proxy_path: String,
    pub(crate) phasing_qc_path: String,
    pub(crate) phasing_manifest_path: String,
    pub(crate) logs_path: String,
    pub(crate) stage_output_ids: Vec<String>,
    pub(crate) raw_output_ids: Vec<String>,
    pub(crate) parser_output_ids: Vec<String>,
    pub(crate) required_inputs: Vec<VcfPhasingFamilyAdapterArtifact>,
    pub(crate) declared_outputs: Vec<VcfPhasingFamilyAdapterArtifact>,
    pub(crate) command_steps: Vec<VcfPhasingFamilyAdapterCommandStep>,
    pub(crate) argv_validation_passed: bool,
    pub(crate) missing_input_probe_artifact_id: String,
    pub(crate) missing_input_expected_error_fragment: String,
    pub(crate) missing_input_observed_error: String,
    pub(crate) missing_input_test_passed: bool,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfPhasingFamilyAdapterReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) tool_id: String,
    pub(crate) tool_status: String,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) parser_output_row_count: usize,
    pub(crate) indexed_row_count: usize,
    pub(crate) missing_input_test_passed_row_count: usize,
    pub(crate) rows: Vec<VcfPhasingFamilyAdapterRow>,
}

#[derive(Debug, Clone)]
struct RegistryToolContract {
    tool_id: String,
    tool_status: String,
    stage_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct MaterializedPanelInputs {
    panel_id: String,
    map_id: String,
    panel_vcf_path: String,
    genetic_map_path: String,
}

struct StageContract {
    command_contract_source: String,
    phased_vcf_path: String,
    phased_vcf_tbi_path: String,
    phase_block_stats_path: String,
    switch_error_proxy_path: String,
    phasing_qc_path: String,
    phasing_manifest_path: String,
    logs_path: String,
    raw_output_ids: Vec<String>,
    parser_output_ids: Vec<String>,
    declared_outputs: Vec<VcfPhasingFamilyAdapterArtifact>,
    command_steps: Vec<VcfPhasingFamilyAdapterCommandStep>,
}

pub(crate) fn run_render_vcf_shapeit5_adapter(
    args: &parse::BenchReadinessRenderVcfShapeit5AdapterArgs,
) -> Result<()> {
    run_render_tool_adapter(
        "shapeit5",
        args.output.clone(),
        args.json,
        DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH,
    )
}

pub(crate) fn run_render_vcf_eagle_adapter(
    args: &parse::BenchReadinessRenderVcfEagleAdapterArgs,
) -> Result<()> {
    run_render_tool_adapter("eagle", args.output.clone(), args.json, DEFAULT_VCF_EAGLE_ADAPTER_PATH)
}

pub(crate) fn run_render_vcf_beagle_adapter(
    args: &parse::BenchReadinessRenderVcfBeagleAdapterArgs,
) -> Result<()> {
    run_render_tool_adapter(
        "beagle",
        args.output.clone(),
        args.json,
        DEFAULT_VCF_BEAGLE_ADAPTER_PATH,
    )
}

fn run_render_tool_adapter(
    tool_id: &str,
    output: Option<PathBuf>,
    json: bool,
    default_output_path: &str,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_phasing_family_adapter(
        &repo_root,
        tool_id,
        output.unwrap_or_else(|| PathBuf::from(default_output_path)),
    )?;
    if json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_phasing_family_adapter(
    repo_root: &Path,
    tool_id: &str,
    output_path: PathBuf,
) -> Result<VcfPhasingFamilyAdapterReport> {
    let registry_tool = load_registry_tool_contract(repo_root, tool_id)?;
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_vcf_phasing_family_adapter_rows(repo_root, &registry_tool)?;
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let parser_output_row_count =
        rows.iter().filter(|row| !row.parser_output_ids.is_empty()).count();
    let indexed_row_count = rows
        .iter()
        .filter(|row| row.raw_output_ids.iter().any(|item| item == "phased_vcf_tbi"))
        .count();
    let missing_input_test_passed_row_count =
        rows.iter().filter(|row| row.missing_input_test_passed).count();

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let report = VcfPhasingFamilyAdapterReport {
        schema_version: schema_version_for_tool(tool_id),
        domain: "vcf",
        tool_id: registry_tool.tool_id.clone(),
        tool_status: registry_tool.tool_status.clone(),
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        benchmark_ready_row_count,
        parser_output_row_count,
        indexed_row_count,
        missing_input_test_passed_row_count,
        rows,
    };
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    Ok(report)
}

fn collect_vcf_phasing_family_adapter_rows(
    repo_root: &Path,
    registry_tool: &RegistryToolContract,
) -> Result<Vec<VcfPhasingFamilyAdapterRow>> {
    let catalog_by_stage = build_vcf_stage_catalog_rows()?
        .into_iter()
        .map(|row| (row.stage_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let matrix_by_stage = build_vcf_stage_matrix_rows()?
        .into_iter()
        .filter(|row| row.tool_id == registry_tool.tool_id)
        .map(|row| (row.stage_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::new();
    for stage_id in &registry_tool.stage_ids {
        let catalog_row = catalog_by_stage.get(stage_id.as_str()).ok_or_else(|| {
            anyhow!(
                "VCF {} adapter report is missing catalog coverage for `{stage_id}`",
                registry_tool.tool_id
            )
        })?;
        rows.push(build_phasing_family_row(
            repo_root,
            registry_tool,
            stage_id,
            catalog_row,
            matrix_by_stage.get(stage_id.as_str()),
        )?);
    }
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    ensure_vcf_phasing_family_adapter_contract(registry_tool, &rows)?;
    Ok(rows)
}

fn build_phasing_family_row(
    repo_root: &Path,
    registry_tool: &RegistryToolContract,
    stage_id: &str,
    catalog_row: &VcfStageCatalogRow,
    matrix_row: Option<&VcfStageMatrixRow>,
) -> Result<VcfPhasingFamilyAdapterRow> {
    let stage = VcfDomainStage::try_from(stage_id)
        .map_err(|error| anyhow!("unknown VCF stage `{stage_id}`: {error}"))?;
    let adapter_id = matrix_row
        .map(|row| row.adapter_id.clone())
        .or_else(|| vcf_domain_stage_adapter_id(stage).map(str::to_string))
        .ok_or_else(|| {
            anyhow!("VCF {stage_id} / {} row is missing adapter id", registry_tool.tool_id)
        })?;
    let parser_id = matrix_row
        .map(|row| row.parser_id.clone())
        .or_else(|| vcf_domain_stage_parser_id(stage).map(str::to_string))
        .ok_or_else(|| {
            anyhow!("VCF {stage_id} / {} row is missing parser id", registry_tool.tool_id)
        })?;
    let corpus_id = matrix_row
        .map(|row| row.corpus_id.clone())
        .unwrap_or_else(|| "vcf_production_regression".to_string());
    let asset_profile_id = matrix_row
        .map(|row| row.asset_profile_id.clone())
        .unwrap_or_else(|| "vcf_cohort_with_panel".to_string());
    let output_root =
        format!("benchmarks/readiness/adapters/{}/{}", registry_tool.tool_id, stage_id);
    let output_prefix = format!("{output_root}/phased");
    let materialized_inputs = materialize_panel_inputs(repo_root, &output_root)?;
    let fixture_inputs = load_governed_vcf_fixture_inputs(repo_root)?;
    let (indexed_input_vcf_path, indexed_input_vcf_tbi_path) = materialize_indexed_vcf_input(
        repo_root,
        &fixture_inputs.multisample_vcf_path,
        &PathBuf::from(&output_root).join("artifacts/input"),
        "phasing_input.vcf.gz",
    )?;
    let required_inputs = vec![
        artifact("vcf", "variant", &indexed_input_vcf_path),
        artifact("vcf_index", "index", &indexed_input_vcf_tbi_path),
        artifact("reference_panel_vcf", "variant", &materialized_inputs.panel_vcf_path),
        artifact("genetic_map_tsv", "reference", &materialized_inputs.genetic_map_path),
    ];
    validate_required_inputs(repo_root, stage_id, &required_inputs)?;
    let stage_output_ids = vcf_domain_stage_expected_output_ids(stage)
        .ok_or_else(|| anyhow!("VCF {stage_id} is missing expected output ids"))?
        .iter()
        .map(|output| (*output).to_string())
        .collect::<Vec<_>>();
    let contract = build_stage_contract(
        registry_tool.tool_id.as_str(),
        &output_root,
        &output_prefix,
        &materialized_inputs,
    )?;
    let argv_validation_passed =
        validate_command_steps(registry_tool.tool_id.as_str(), stage_id, &contract.command_steps)
            .is_ok();
    let (
        missing_input_probe_artifact_id,
        missing_input_expected_error_fragment,
        missing_input_observed_error,
        missing_input_test_passed,
    ) = run_missing_input_probe(repo_root, stage_id, &required_inputs);

    let reason = format!(
        "row `{stage_id}` / `{}` renders a concrete phasing command with explicit panel `{}`, map `{}`, indexed phased output `{}`, and parser-visible phasing evidence",
        registry_tool.tool_id,
        materialized_inputs.panel_id,
        materialized_inputs.map_id,
        contract.phased_vcf_path
    );

    Ok(VcfPhasingFamilyAdapterRow {
        tool_id: registry_tool.tool_id.clone(),
        tool_status: registry_tool.tool_status.clone(),
        stage_id: stage_id.to_string(),
        stage_status: catalog_row.support_status.clone(),
        benchmark_status: if matrix_row.is_some() {
            "benchmark_ready".to_string()
        } else {
            "not_benchmark_ready".to_string()
        },
        adapter_id,
        parser_id,
        corpus_id,
        asset_profile_id,
        command_contract_source: contract.command_contract_source,
        output_root,
        output_prefix,
        panel_id: materialized_inputs.panel_id,
        map_id: materialized_inputs.map_id,
        input_vcf_path: indexed_input_vcf_path,
        panel_vcf_path: materialized_inputs.panel_vcf_path,
        genetic_map_path: materialized_inputs.genetic_map_path,
        phased_vcf_path: contract.phased_vcf_path,
        phased_vcf_tbi_path: contract.phased_vcf_tbi_path,
        phase_block_stats_path: contract.phase_block_stats_path,
        switch_error_proxy_path: contract.switch_error_proxy_path,
        phasing_qc_path: contract.phasing_qc_path,
        phasing_manifest_path: contract.phasing_manifest_path,
        logs_path: contract.logs_path,
        stage_output_ids,
        raw_output_ids: contract.raw_output_ids,
        parser_output_ids: contract.parser_output_ids,
        required_inputs,
        declared_outputs: contract.declared_outputs,
        command_steps: contract.command_steps,
        argv_validation_passed,
        missing_input_probe_artifact_id,
        missing_input_expected_error_fragment,
        missing_input_observed_error,
        missing_input_test_passed,
        reason,
    })
}

fn build_stage_contract(
    tool_id: &str,
    output_root: &str,
    output_prefix: &str,
    materialized_inputs: &MaterializedPanelInputs,
) -> Result<StageContract> {
    let phased_vcf_path = format!("{output_root}/phased.vcf.gz");
    let phased_vcf_tbi_path = format!("{phased_vcf_path}.tbi");
    let phase_block_stats_path = format!("{output_root}/phase_block_stats.tsv");
    let switch_error_proxy_path = format!("{output_root}/switch_error_proxy.tsv");
    let phasing_qc_path = format!("{output_root}/phasing_qc.json");
    let phasing_manifest_path = format!("{output_root}/phasing_manifest.json");
    let logs_path = format!("{output_root}/logs.txt");
    let (command_contract_source, phase_step_argv) = match tool_id {
        "shapeit5" => (
            "domain/vcf/fixtures/vcf.phasing/shapeit5.txt".to_string(),
            vec![
                "sh".to_string(),
                "-lc".to_string(),
                format!(
                    "shapeit5 phase_common --input '{}' --reference '{}' --map '{}' --region 1:1-1000000 --thread 8 --seed 42 --output '{}' > '{}' 2>&1",
                    GOVERNED_COHORT_VCF_PATH,
                    materialized_inputs.panel_vcf_path,
                    materialized_inputs.genetic_map_path,
                    phased_vcf_path,
                    logs_path
                ),
            ],
        ),
        "eagle" => (
            "domain/vcf/fixtures/vcf.phasing/eagle.txt".to_string(),
            vec![
                "sh".to_string(),
                "-lc".to_string(),
                format!(
                    "eagle --vcfTarget '{}' --vcfRef '{}' --geneticMapFile '{}' --outPrefix '{}' --numThreads 8 > '{}' 2>&1",
                    GOVERNED_COHORT_VCF_PATH,
                    materialized_inputs.panel_vcf_path,
                    materialized_inputs.genetic_map_path,
                    output_prefix,
                    logs_path
                ),
            ],
        ),
        "beagle" => (
            "domain/vcf/fixtures/vcf.phasing/beagle.txt".to_string(),
            vec![
                "sh".to_string(),
                "-lc".to_string(),
                format!(
                    "beagle gt='{}' ref='{}' map='{}' out='{}' nthreads=8 seed=42 > '{}' 2>&1",
                    GOVERNED_COHORT_VCF_PATH,
                    materialized_inputs.panel_vcf_path,
                    materialized_inputs.genetic_map_path,
                    output_prefix,
                    logs_path
                ),
            ],
        ),
        other => return Err(anyhow!("VCF phasing-family adapter does not govern tool `{other}`")),
    };

    Ok(StageContract {
        command_contract_source,
        phased_vcf_path: phased_vcf_path.clone(),
        phased_vcf_tbi_path: phased_vcf_tbi_path.clone(),
        phase_block_stats_path: phase_block_stats_path.clone(),
        switch_error_proxy_path: switch_error_proxy_path.clone(),
        phasing_qc_path: phasing_qc_path.clone(),
        phasing_manifest_path: phasing_manifest_path.clone(),
        logs_path: logs_path.clone(),
        raw_output_ids: vec![
            "phased_vcf".to_string(),
            "phased_vcf_tbi".to_string(),
            "phase_block_stats".to_string(),
            "switch_error_proxy".to_string(),
            "phasing_log".to_string(),
        ],
        parser_output_ids: vec!["phasing_qc".to_string(), "phasing_manifest".to_string()],
        declared_outputs: vec![
            artifact("phased_vcf", "variant", &phased_vcf_path),
            artifact("phased_vcf_tbi", "index", &phased_vcf_tbi_path),
            artifact("phase_block_stats", "report_tsv", &phase_block_stats_path),
            artifact("switch_error_proxy", "report_tsv", &switch_error_proxy_path),
            artifact("phasing_qc", "report_json", &phasing_qc_path),
            artifact("phasing_manifest", "report_json", &phasing_manifest_path),
            artifact("phasing_log", "log", &logs_path),
        ],
        command_steps: vec![
            VcfPhasingFamilyAdapterCommandStep {
                step_id: "phase".to_string(),
                step_kind: "phasing".to_string(),
                argv: phase_step_argv,
                declared_output_artifact_ids: vec![
                    "phased_vcf".to_string(),
                    "phasing_log".to_string(),
                ],
            },
            step(
                "index_phased_vcf",
                "index",
                vec!["bcftools", "index", "-t", &phased_vcf_path],
                &["phased_vcf_tbi"],
            ),
        ],
    })
}

fn materialize_panel_inputs(
    repo_root: &Path,
    output_root: &str,
) -> Result<MaterializedPanelInputs> {
    let materialization_root = repo_root.join(output_root).join("artifacts/reference");
    if materialization_root.exists() {
        fs::remove_dir_all(&materialization_root)
            .with_context(|| format!("remove {}", materialization_root.display()))?;
    }
    let report = materialize_governed_vcf_panel_assets(&materialization_root)?;
    let panel_vcf_path = report
        .materialized_files
        .iter()
        .find(|path| path.ends_with("/panel.vcf.gz"))
        .cloned()
        .ok_or_else(|| {
            anyhow!("governed VCF panel materialization did not produce panel.vcf.gz")
        })?;
    let genetic_map_path = report
        .materialized_files
        .iter()
        .find(|path| path.ends_with("/recombination_map.tsv.gz"))
        .cloned()
        .ok_or_else(|| {
            anyhow!("governed VCF panel materialization did not produce recombination_map.tsv.gz")
        })?;

    Ok(MaterializedPanelInputs {
        panel_id: report.panel_id,
        map_id: report.map_id,
        panel_vcf_path: path_relative_to_repo(repo_root, Path::new(&panel_vcf_path)),
        genetic_map_path: path_relative_to_repo(repo_root, Path::new(&genetic_map_path)),
    })
}

fn load_registry_tool_contract(repo_root: &Path, tool_id: &str) -> Result<RegistryToolContract> {
    let registry_path = repo_root.join("configs/ci/registry/tool_registry_vcf_downstream.toml");
    let raw = fs::read_to_string(&registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let parsed: toml::Value =
        toml::from_str(&raw).with_context(|| format!("parse {}", registry_path.display()))?;
    let tools = parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow!("missing tools in {}", registry_path.display()))?;
    let tool = tools
        .iter()
        .find(|entry| {
            entry
                .get("tool_id")
                .and_then(toml::Value::as_str)
                .is_some_and(|candidate| candidate == tool_id)
        })
        .ok_or_else(|| {
            anyhow!("missing {tool_id} VCF registry row in {}", registry_path.display())
        })?;
    let tool_status = tool
        .get("status")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| anyhow!("{tool_id} VCF registry row is missing status"))?;
    let stage_ids = tool
        .get("stage_ids")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow!("{tool_id} VCF registry row is missing stage_ids"))?
        .iter()
        .filter_map(toml::Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    Ok(RegistryToolContract {
        tool_id: tool_id.to_string(),
        tool_status: tool_status.to_string(),
        stage_ids,
    })
}

fn schema_version_for_tool(tool_id: &str) -> &'static str {
    match tool_id {
        "shapeit5" => VCF_SHAPEIT5_ADAPTER_SCHEMA_VERSION,
        "eagle" => VCF_EAGLE_ADAPTER_SCHEMA_VERSION,
        "beagle" => VCF_BEAGLE_ADAPTER_SCHEMA_VERSION,
        other => panic!("unsupported VCF phasing-family schema tool `{other}`"),
    }
}

fn validate_required_inputs(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfPhasingFamilyAdapterArtifact],
) -> Result<()> {
    for input in inputs {
        let resolved = repo_root.join(&input.path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "VCF phasing-family adapter for `{stage_id}` is missing required input `{}` at `{}`",
                input.artifact_id,
                input.path
            ));
        }
    }
    Ok(())
}

fn validate_command_steps(
    tool_id: &str,
    stage_id: &str,
    steps: &[VcfPhasingFamilyAdapterCommandStep],
) -> Result<()> {
    if steps.is_empty() {
        return Err(anyhow!(
            "VCF {tool_id} adapter row `{stage_id}` must declare at least one command step"
        ));
    }
    let joined =
        steps.iter().flat_map(|step| step.argv.iter().cloned()).collect::<Vec<_>>().join(" ");
    if joined.contains("--help") || joined.to_ascii_lowercase().contains("placeholder") {
        return Err(anyhow!(
            "VCF {tool_id} adapter row `{stage_id}` still contains placeholder argv: {joined}"
        ));
    }
    for needle in ["panel", "map", "bcftools index", ".vcf.gz"] {
        if !joined.contains(needle) {
            return Err(anyhow!(
                "VCF {tool_id} adapter row `{stage_id}` is missing required token `{needle}`: {joined}"
            ));
        }
    }
    match tool_id {
        "shapeit5" => {
            for needle in ["shapeit5", "phase_common", "--reference", "--map", "--output"] {
                if !joined.contains(needle) {
                    return Err(anyhow!(
                        "VCF shapeit5 adapter row `{stage_id}` is missing `{needle}`: {joined}"
                    ));
                }
            }
        }
        "eagle" => {
            for needle in ["eagle", "--vcfTarget", "--vcfRef", "--geneticMapFile", "--outPrefix"] {
                if !joined.contains(needle) {
                    return Err(anyhow!(
                        "VCF eagle adapter row `{stage_id}` is missing `{needle}`: {joined}"
                    ));
                }
            }
        }
        "beagle" => {
            for needle in ["beagle", "gt=", "ref=", "map=", "out="] {
                if !joined.contains(needle) {
                    return Err(anyhow!(
                        "VCF beagle adapter row `{stage_id}` is missing `{needle}`: {joined}"
                    ));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn run_missing_input_probe(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfPhasingFamilyAdapterArtifact],
) -> (String, String, String, bool) {
    let mut mutated_inputs = inputs.to_vec();
    let probe = mutated_inputs.first().cloned().unwrap_or_else(|| {
        artifact("missing_input", "unknown", "artifacts/bench-readiness/adapters/none.missing")
    });
    if let Some(first) = mutated_inputs.first_mut() {
        first.path = format!(
            "artifacts/bench-readiness/adapters/probes/{stage_id}/{}.missing",
            first.artifact_id
        );
    }
    let expected_error_fragment = format!("required input `{}`", probe.artifact_id);
    let observed_error = match validate_required_inputs(repo_root, stage_id, &mutated_inputs) {
        Ok(()) => format!(
            "VCF phasing-family adapter row `{stage_id}` unexpectedly accepted missing input `{}`",
            probe.artifact_id
        ),
        Err(error) => error.to_string(),
    };
    let passed = observed_error.contains(&expected_error_fragment);
    (probe.artifact_id, expected_error_fragment, observed_error, passed)
}

fn ensure_vcf_phasing_family_adapter_contract(
    registry_tool: &RegistryToolContract,
    rows: &[VcfPhasingFamilyAdapterRow],
) -> Result<()> {
    let expected_stage_ids =
        registry_tool.stage_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let observed_stage_ids = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
    if observed_stage_ids != expected_stage_ids {
        return Err(anyhow!(
            "VCF {} adapter stage set drifted: expected {:?}, found {:?}",
            registry_tool.tool_id,
            expected_stage_ids,
            observed_stage_ids
        ));
    }
    for row in rows {
        if row.tool_status != registry_tool.tool_status {
            return Err(anyhow!(
                "VCF {} adapter row `{}` drifted from registry tool status `{}`: {}",
                registry_tool.tool_id,
                row.stage_id,
                registry_tool.tool_status,
                row.tool_status
            ));
        }
        if !row.argv_validation_passed {
            return Err(anyhow!(
                "VCF {} adapter row `{}` failed argv validation",
                registry_tool.tool_id,
                row.stage_id
            ));
        }
        if !row.missing_input_test_passed {
            return Err(anyhow!(
                "VCF {} adapter row `{}` failed missing-input validation: {}",
                registry_tool.tool_id,
                row.stage_id,
                row.missing_input_observed_error
            ));
        }
        if row.parser_output_ids.is_empty() {
            return Err(anyhow!(
                "VCF {} adapter row `{}` is missing parser output declarations",
                registry_tool.tool_id,
                row.stage_id
            ));
        }
    }
    Ok(())
}

fn artifact(artifact_id: &str, role: &str, path: &str) -> VcfPhasingFamilyAdapterArtifact {
    VcfPhasingFamilyAdapterArtifact {
        artifact_id: artifact_id.to_string(),
        role: role.to_string(),
        path: path.to_string(),
    }
}

fn step(
    step_id: &str,
    step_kind: &str,
    argv: Vec<&str>,
    declared_output_artifact_ids: &[&str],
) -> VcfPhasingFamilyAdapterCommandStep {
    VcfPhasingFamilyAdapterCommandStep {
        step_id: step_id.to_string(),
        step_kind: step_kind.to_string(),
        argv: argv.into_iter().map(str::to_string).collect(),
        declared_output_artifact_ids: declared_output_artifact_ids
            .iter()
            .map(|artifact_id| (*artifact_id).to_string())
            .collect(),
    }
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
        render_vcf_phasing_family_adapter, DEFAULT_VCF_BEAGLE_ADAPTER_PATH,
        DEFAULT_VCF_EAGLE_ADAPTER_PATH, DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_shapeit5_adapter_tracks_governed_rows() {
        let report = render_vcf_phasing_family_adapter(
            &repo_root(),
            "shapeit5",
            PathBuf::from(DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH),
        )
        .expect("render shapeit5 adapter");
        assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_shapeit5_adapter.v1");
        assert_eq!(report.tool_id, "shapeit5");
        assert_eq!(report.row_count, 1);
        assert_eq!(report.benchmark_ready_row_count, 1);
        assert_eq!(report.indexed_row_count, 1);
    }

    #[test]
    fn vcf_eagle_adapter_tracks_governed_rows() {
        let report = render_vcf_phasing_family_adapter(
            &repo_root(),
            "eagle",
            PathBuf::from(DEFAULT_VCF_EAGLE_ADAPTER_PATH),
        )
        .expect("render eagle adapter");
        assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_eagle_adapter.v1");
        assert_eq!(report.tool_id, "eagle");
        assert_eq!(report.row_count, 1);
        assert_eq!(report.benchmark_ready_row_count, 0);
        assert_eq!(report.indexed_row_count, 1);
    }

    #[test]
    fn vcf_beagle_adapter_tracks_governed_rows() {
        let report = render_vcf_phasing_family_adapter(
            &repo_root(),
            "beagle",
            PathBuf::from(DEFAULT_VCF_BEAGLE_ADAPTER_PATH),
        )
        .expect("render beagle adapter");
        assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_beagle_adapter.v1");
        assert_eq!(report.tool_id, "beagle");
        assert_eq!(report.row_count, 1);
        assert_eq!(report.benchmark_ready_row_count, 0);
        assert_eq!(report.indexed_row_count, 1);
    }
}
