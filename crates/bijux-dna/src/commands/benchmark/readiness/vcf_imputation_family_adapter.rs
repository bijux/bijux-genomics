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
use crate::commands::benchmark::readiness::vcf_readiness_inputs::materialize_indexed_vcf_input;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH: &str =
    "target/bench-readiness/adapters/imputation-family.vcf.json";
const VCF_IMPUTATION_FAMILY_ADAPTER_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_imputation_family_adapter.v1";
const GOVERNED_IMPUTATION_STAGE_IDS: [&str; 2] = ["vcf.imputation", "vcf.impute"];
const GOVERNED_IMPUTATION_TOOL_IDS: [&str; 4] = ["beagle", "glimpse", "impute5", "minimac4"];
const GOVERNED_GTCOHORT_VCF_PATH: &str =
    "tests/fixtures/corpora/vcf-mini/variants/vcf_mini_phased.vcf";
const GOVERNED_GLLIKE_VCF_PATH: &str =
    "tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf";
const GOVERNED_REGION_LITERAL: &str = "1:1-1000000";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfImputationFamilyAdapterArtifact {
    pub(crate) artifact_id: String,
    pub(crate) role: String,
    pub(crate) path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfImputationFamilyAdapterCommandStep {
    pub(crate) step_id: String,
    pub(crate) step_kind: String,
    pub(crate) argv: Vec<String>,
    pub(crate) declared_output_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfImputationFamilyAdapterRow {
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
    pub(crate) target_vcf_path: String,
    pub(crate) panel_vcf_path: String,
    pub(crate) panel_m3vcf_path: Option<String>,
    pub(crate) genetic_map_path: String,
    pub(crate) region_literal: Option<String>,
    pub(crate) imputed_vcf_path: String,
    pub(crate) imputed_vcf_tbi_path: String,
    pub(crate) quality_output_path: String,
    pub(crate) quality_tsv_path: Option<String>,
    pub(crate) warnings_path: Option<String>,
    pub(crate) imputation_manifest_path: String,
    pub(crate) orchestration_manifest_path: Option<String>,
    pub(crate) panel_mismatch_diagnostics_path: Option<String>,
    pub(crate) log_output_path: String,
    pub(crate) stage_output_ids: Vec<String>,
    pub(crate) raw_output_ids: Vec<String>,
    pub(crate) parser_output_ids: Vec<String>,
    pub(crate) required_inputs: Vec<VcfImputationFamilyAdapterArtifact>,
    pub(crate) declared_outputs: Vec<VcfImputationFamilyAdapterArtifact>,
    pub(crate) command_steps: Vec<VcfImputationFamilyAdapterCommandStep>,
    pub(crate) argv_validation_passed: bool,
    pub(crate) missing_input_probe_artifact_id: String,
    pub(crate) missing_input_expected_error_fragment: String,
    pub(crate) missing_input_observed_error: String,
    pub(crate) missing_input_test_passed: bool,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfImputationFamilyAdapterReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) parser_output_row_count: usize,
    pub(crate) missing_input_test_passed_row_count: usize,
    pub(crate) rows: Vec<VcfImputationFamilyAdapterRow>,
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
    panel_m3vcf_path: Option<String>,
    genetic_map_path: String,
}

struct StageContract {
    command_contract_source: String,
    imputed_vcf_path: String,
    imputed_vcf_tbi_path: String,
    quality_output_path: String,
    quality_tsv_path: Option<String>,
    warnings_path: Option<String>,
    imputation_manifest_path: String,
    orchestration_manifest_path: Option<String>,
    panel_mismatch_diagnostics_path: Option<String>,
    log_output_path: String,
    raw_output_ids: Vec<String>,
    parser_output_ids: Vec<String>,
    declared_outputs: Vec<VcfImputationFamilyAdapterArtifact>,
    command_steps: Vec<VcfImputationFamilyAdapterCommandStep>,
}

pub(crate) fn run_render_vcf_imputation_family_adapter(
    args: &parse::BenchReadinessRenderVcfImputationFamilyAdapterArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_imputation_family_adapter(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_imputation_family_adapter(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfImputationFamilyAdapterReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_vcf_imputation_family_adapter_rows(repo_root)?;
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let parser_output_row_count =
        rows.iter().filter(|row| !row.parser_output_ids.is_empty()).count();
    let missing_input_test_passed_row_count =
        rows.iter().filter(|row| row.missing_input_test_passed).count();

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let report = VcfImputationFamilyAdapterReport {
        schema_version: VCF_IMPUTATION_FAMILY_ADAPTER_SCHEMA_VERSION,
        domain: "vcf",
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        tool_count,
        benchmark_ready_row_count,
        parser_output_row_count,
        missing_input_test_passed_row_count,
        rows,
    };
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    Ok(report)
}

fn collect_vcf_imputation_family_adapter_rows(
    repo_root: &Path,
) -> Result<Vec<VcfImputationFamilyAdapterRow>> {
    let catalog_by_stage = build_vcf_stage_catalog_rows()?
        .into_iter()
        .map(|row| (row.stage_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let matrix_by_pair = build_vcf_stage_matrix_rows()?
        .into_iter()
        .map(|row| ((row.stage_id.clone(), row.tool_id.clone()), row))
        .collect::<BTreeMap<_, _>>();

    let mut registry_by_tool = BTreeMap::<String, RegistryToolContract>::new();
    for tool_id in GOVERNED_IMPUTATION_TOOL_IDS {
        registry_by_tool
            .insert(tool_id.to_string(), load_registry_tool_contract(repo_root, tool_id)?);
    }

    let mut rows = Vec::new();
    for tool_id in GOVERNED_IMPUTATION_TOOL_IDS {
        let registry_tool = registry_by_tool
            .get(tool_id)
            .ok_or_else(|| anyhow!("missing governed registry tool `{tool_id}`"))?;
        for stage_id in GOVERNED_IMPUTATION_STAGE_IDS {
            let catalog_row = catalog_by_stage.get(stage_id).ok_or_else(|| {
                anyhow!(
                    "VCF imputation-family adapter report is missing catalog coverage for `{stage_id}`"
                )
            })?;
            rows.push(build_imputation_family_row(
                repo_root,
                registry_tool,
                stage_id,
                catalog_row,
                matrix_by_pair.get(&(stage_id.to_string(), tool_id.to_string())),
            )?);
        }
    }
    rows.sort_by(|left, right| {
        left.tool_id.cmp(&right.tool_id).then(left.stage_id.cmp(&right.stage_id))
    });
    ensure_vcf_imputation_family_adapter_contract(&registry_by_tool, &rows)?;
    Ok(rows)
}

fn build_imputation_family_row(
    repo_root: &Path,
    registry_tool: &RegistryToolContract,
    stage_id: &str,
    catalog_row: &VcfStageCatalogRow,
    matrix_row: Option<&VcfStageMatrixRow>,
) -> Result<VcfImputationFamilyAdapterRow> {
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
    let output_root = format!(
        "target/bench-readiness/adapters/imputation/{}/{}",
        registry_tool.tool_id, stage_id
    );
    let output_prefix = format!("{output_root}/imputed");
    let materialized_inputs = materialize_panel_inputs(repo_root, &output_root)?;
    let (target_vcf_path, target_vcf_index_path) = materialize_indexed_vcf_input(
        repo_root,
        governed_target_vcf_path(registry_tool.tool_id.as_str()),
        &PathBuf::from(&output_root).join("artifacts/input"),
        &format!("{}.vcf.gz", stage_id.replace('.', "_")),
    )?;
    let mut required_inputs = vec![
        artifact("vcf", "variant", &target_vcf_path),
        artifact("vcf_index", "index", &target_vcf_index_path),
        artifact("reference_panel_vcf", "variant", &materialized_inputs.panel_vcf_path),
    ];
    if registry_tool.tool_id == "minimac4" {
        required_inputs.push(artifact(
            "reference_panel_m3vcf",
            "variant",
            materialized_inputs.panel_m3vcf_path.as_deref().ok_or_else(|| {
                anyhow!("VCF minimac4 adapter row `{stage_id}` is missing panel.m3vcf.gz")
            })?,
        ));
    } else {
        required_inputs.push(artifact(
            "genetic_map_tsv",
            "reference",
            &materialized_inputs.genetic_map_path,
        ));
    }
    validate_required_inputs(repo_root, stage_id, &required_inputs)?;
    let stage_output_ids = vcf_domain_stage_expected_output_ids(stage)
        .ok_or_else(|| anyhow!("VCF {stage_id} is missing expected output ids"))?
        .iter()
        .map(|output| (*output).to_string())
        .collect::<Vec<_>>();
    let contract = build_stage_contract(
        registry_tool.tool_id.as_str(),
        stage,
        &target_vcf_path,
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
    let benchmark_status = if matrix_row.is_some() {
        "benchmark_ready".to_string()
    } else {
        "not_benchmark_ready".to_string()
    };
    let region_literal = if matches!(registry_tool.tool_id.as_str(), "glimpse" | "impute5") {
        Some(GOVERNED_REGION_LITERAL.to_string())
    } else {
        None
    };
    let reason = format!(
        "row `{stage_id}` / `{}` renders concrete imputation argv with target `{}`, panel `{}`, quality output `{}`, and log output `{}`",
        registry_tool.tool_id,
        target_vcf_path,
        materialized_inputs.panel_id,
        contract.quality_output_path,
        contract.log_output_path
    );

    Ok(VcfImputationFamilyAdapterRow {
        tool_id: registry_tool.tool_id.clone(),
        tool_status: registry_tool.tool_status.clone(),
        stage_id: stage_id.to_string(),
        stage_status: catalog_row.support_status.clone(),
        benchmark_status,
        adapter_id,
        parser_id,
        corpus_id,
        asset_profile_id,
        command_contract_source: contract.command_contract_source,
        output_root,
        output_prefix,
        panel_id: materialized_inputs.panel_id,
        map_id: materialized_inputs.map_id,
        target_vcf_path,
        panel_vcf_path: materialized_inputs.panel_vcf_path,
        panel_m3vcf_path: materialized_inputs.panel_m3vcf_path,
        genetic_map_path: materialized_inputs.genetic_map_path,
        region_literal,
        imputed_vcf_path: contract.imputed_vcf_path,
        imputed_vcf_tbi_path: contract.imputed_vcf_tbi_path,
        quality_output_path: contract.quality_output_path,
        quality_tsv_path: contract.quality_tsv_path,
        warnings_path: contract.warnings_path,
        imputation_manifest_path: contract.imputation_manifest_path,
        orchestration_manifest_path: contract.orchestration_manifest_path,
        panel_mismatch_diagnostics_path: contract.panel_mismatch_diagnostics_path,
        log_output_path: contract.log_output_path,
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

fn governed_target_vcf_path(tool_id: &str) -> &'static str {
    match tool_id {
        "glimpse" => GOVERNED_GLLIKE_VCF_PATH,
        "beagle" | "impute5" | "minimac4" => GOVERNED_GTCOHORT_VCF_PATH,
        other => panic!("unsupported VCF imputation-family tool `{other}`"),
    }
}

fn build_stage_contract(
    tool_id: &str,
    stage: VcfDomainStage,
    target_vcf_path: &str,
    output_root: &str,
    output_prefix: &str,
    materialized_inputs: &MaterializedPanelInputs,
) -> Result<StageContract> {
    let imputed_vcf_path = format!("{output_root}/imputed.vcf.gz");
    let imputed_vcf_tbi_path = format!("{imputed_vcf_path}.tbi");
    let quality_output_path = format!("{output_root}/imputation_qc.json");
    let quality_tsv_path = if stage == VcfDomainStage::Impute {
        Some(format!("{output_root}/imputation_qc.tsv"))
    } else {
        None
    };
    let warnings_path = if stage == VcfDomainStage::Impute {
        Some(format!("{output_root}/warnings.json"))
    } else {
        None
    };
    let imputation_manifest_path = format!("{output_root}/imputation_manifest.json");
    let orchestration_manifest_path = if stage == VcfDomainStage::Imputation {
        Some(format!("{output_root}/orchestration_manifest.json"))
    } else {
        None
    };
    let panel_mismatch_diagnostics_path = if stage == VcfDomainStage::Impute {
        Some(format!("{output_root}/panel_mismatch_diagnostics.json"))
    } else {
        None
    };
    let log_output_path = format!("{output_root}/logs.txt");
    let (command_contract_source, impute_argv) = match tool_id {
        "beagle" => (
            format!("domain/vcf/fixtures/{}//beagle.txt", stage.as_str()).replace("//", "/"),
            vec![
                "sh".to_string(),
                "-lc".to_string(),
                format!(
                    "beagle gt='{}' ref='{}' map='{}' out='{}' impute=true nthreads=8 seed=42 > '{}' 2>&1",
                    target_vcf_path,
                    materialized_inputs.panel_vcf_path,
                    materialized_inputs.genetic_map_path,
                    output_prefix,
                    log_output_path
                ),
            ],
        ),
        "glimpse" => (
            format!("domain/vcf/fixtures/{}//glimpse.txt", stage.as_str()).replace("//", "/"),
            vec![
                "sh".to_string(),
                "-lc".to_string(),
                format!(
                    "GLIMPSE_phase --input '{}' --reference '{}' --map '{}' --input-region '{}' --output-region '{}' --threads 8 --seed 42 --output '{}' > '{}' 2>&1",
                    target_vcf_path,
                    materialized_inputs.panel_vcf_path,
                    materialized_inputs.genetic_map_path,
                    GOVERNED_REGION_LITERAL,
                    GOVERNED_REGION_LITERAL,
                    imputed_vcf_path,
                    log_output_path
                ),
            ],
        ),
        "impute5" => (
            format!("domain/vcf/fixtures/{}//impute5.txt", stage.as_str()).replace("//", "/"),
            vec![
                "sh".to_string(),
                "-lc".to_string(),
                format!(
                    "impute5 --g '{}' --h '{}' --m '{}' --r '{}' --o '{}' --threads 8 --seed 42 > '{}' 2>&1",
                    target_vcf_path,
                    materialized_inputs.panel_vcf_path,
                    materialized_inputs.genetic_map_path,
                    GOVERNED_REGION_LITERAL,
                    imputed_vcf_path,
                    log_output_path
                ),
            ],
        ),
        "minimac4" => (
            format!("domain/vcf/fixtures/{}//minimac4.txt", stage.as_str()).replace("//", "/"),
            vec![
                "sh".to_string(),
                "-lc".to_string(),
                format!(
                    "minimac4 --refHaps '{}' --haps '{}' --prefix '{}' --cpus 8 > '{}' 2>&1 && mv '{}.dose.vcf.gz' '{}'",
                    materialized_inputs
                        .panel_m3vcf_path
                        .as_deref()
                        .ok_or_else(|| anyhow!("missing panel.m3vcf.gz for minimac4"))?,
                    target_vcf_path,
                    output_prefix,
                    log_output_path,
                    output_prefix,
                    imputed_vcf_path
                ),
            ],
        ),
        other => {
            return Err(anyhow!(
                "VCF imputation-family adapter does not govern tool `{other}`"
            ));
        }
    };

    let mut raw_output_ids = vec![
        "imputed_vcf".to_string(),
        "imputed_tbi".to_string(),
        "imputation_qc_json".to_string(),
        "imputation_accept_json".to_string(),
        "imputation_manifest_json".to_string(),
        "logs_txt".to_string(),
    ];
    if stage == VcfDomainStage::Impute {
        raw_output_ids.extend([
            "imputation_qc_tsv".to_string(),
            "maf_bin_quality_tsv".to_string(),
            "info_hist_json".to_string(),
            "warnings_json".to_string(),
            "overlap_stats_json".to_string(),
            "panel_mismatch_diagnostics_json".to_string(),
        ]);
    } else {
        raw_output_ids.push("orchestration_manifest_json".to_string());
    }

    let parser_output_ids = if stage == VcfDomainStage::Impute {
        vec![
            "imputation_qc".to_string(),
            "imputation_accept".to_string(),
            "imputation_manifest".to_string(),
            "panel_mismatch_diagnostics".to_string(),
        ]
    } else {
        vec![
            "imputation_qc".to_string(),
            "imputation_accept".to_string(),
            "imputation_manifest".to_string(),
            "orchestration_manifest".to_string(),
        ]
    };

    let mut declared_outputs = vec![
        artifact("imputed_vcf", "variant", &imputed_vcf_path),
        artifact("imputed_tbi", "index", &imputed_vcf_tbi_path),
        artifact("imputation_qc_json", "report_json", &quality_output_path),
        artifact(
            "imputation_accept_json",
            "report_json",
            &format!("{output_root}/imputation_accept.json"),
        ),
        artifact("imputation_manifest_json", "report_json", &imputation_manifest_path),
        artifact("logs_txt", "log", &log_output_path),
    ];
    if let Some(path) = &quality_tsv_path {
        declared_outputs.push(artifact("imputation_qc_tsv", "report_tsv", path));
        declared_outputs.push(artifact(
            "maf_bin_quality_tsv",
            "report_tsv",
            &format!("{output_root}/maf_bins.tsv"),
        ));
        declared_outputs.push(artifact(
            "info_hist_json",
            "report_json",
            &format!("{output_root}/info_hist.json"),
        ));
        declared_outputs.push(artifact(
            "warnings_json",
            "report_json",
            warnings_path.as_deref().expect("warnings path for vcf.impute"),
        ));
        declared_outputs.push(artifact(
            "overlap_stats_json",
            "report_json",
            &format!("{output_root}/overlap_stats.json"),
        ));
        declared_outputs.push(artifact(
            "panel_mismatch_diagnostics_json",
            "report_json",
            panel_mismatch_diagnostics_path.as_deref().expect("panel mismatch path for vcf.impute"),
        ));
    }
    if let Some(path) = &orchestration_manifest_path {
        declared_outputs.push(artifact("orchestration_manifest_json", "report_json", path));
    }

    Ok(StageContract {
        command_contract_source,
        imputed_vcf_path: imputed_vcf_path.clone(),
        imputed_vcf_tbi_path: imputed_vcf_tbi_path.clone(),
        quality_output_path: quality_output_path.clone(),
        quality_tsv_path,
        warnings_path,
        imputation_manifest_path: imputation_manifest_path.clone(),
        orchestration_manifest_path,
        panel_mismatch_diagnostics_path,
        log_output_path: log_output_path.clone(),
        raw_output_ids,
        parser_output_ids,
        declared_outputs,
        command_steps: vec![
            VcfImputationFamilyAdapterCommandStep {
                step_id: "impute".to_string(),
                step_kind: "imputation".to_string(),
                argv: impute_argv,
                declared_output_artifact_ids: vec![
                    "imputed_vcf".to_string(),
                    "logs_txt".to_string(),
                ],
            },
            step(
                "index_imputed_vcf",
                "index",
                vec!["bcftools", "index", "-t", &imputed_vcf_path],
                &["imputed_tbi"],
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
    let panel_m3vcf_path =
        report.materialized_files.iter().find(|path| path.ends_with("/panel.m3vcf.gz")).cloned();
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
        panel_m3vcf_path: panel_m3vcf_path
            .as_deref()
            .map(|path| path_relative_to_repo(repo_root, Path::new(path))),
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
    let required_stage_ids = GOVERNED_IMPUTATION_STAGE_IDS.iter().copied().collect::<BTreeSet<_>>();
    let tool = tools
        .iter()
        .find(|entry| {
            let Some(candidate) = entry.get("tool_id").and_then(toml::Value::as_str) else {
                return false;
            };
            if candidate != tool_id {
                return false;
            }
            let stage_ids = entry
                .get("stage_ids")
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(toml::Value::as_str)
                .collect::<BTreeSet<_>>();
            stage_ids.is_superset(&required_stage_ids)
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

fn validate_required_inputs(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfImputationFamilyAdapterArtifact],
) -> Result<()> {
    for input in inputs {
        let resolved = repo_root.join(&input.path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "VCF imputation-family adapter for `{stage_id}` is missing required input `{}` at `{}`",
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
    steps: &[VcfImputationFamilyAdapterCommandStep],
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
    if !joined.contains("bcftools index") || !joined.contains(".vcf.gz") {
        return Err(anyhow!(
            "VCF {tool_id} adapter row `{stage_id}` must keep indexed VCF output wiring: {joined}"
        ));
    }
    match tool_id {
        "beagle" => {
            for needle in ["beagle", "gt=", "ref=", "map=", "impute=true"] {
                if !joined.contains(needle) {
                    return Err(anyhow!(
                        "VCF beagle adapter row `{stage_id}` is missing `{needle}`: {joined}"
                    ));
                }
            }
        }
        "glimpse" => {
            for needle in
                ["GLIMPSE_phase", "--reference", "--map", "--input-region", "--output-region"]
            {
                if !joined.contains(needle) {
                    return Err(anyhow!(
                        "VCF glimpse adapter row `{stage_id}` is missing `{needle}`: {joined}"
                    ));
                }
            }
        }
        "impute5" => {
            for needle in ["impute5", "--g", "--h", "--m", "--r"] {
                if !joined.contains(needle) {
                    return Err(anyhow!(
                        "VCF impute5 adapter row `{stage_id}` is missing `{needle}`: {joined}"
                    ));
                }
            }
        }
        "minimac4" => {
            for needle in ["minimac4", "--refHaps", "panel.m3vcf.gz", "--prefix"] {
                if !joined.contains(needle) {
                    return Err(anyhow!(
                        "VCF minimac4 adapter row `{stage_id}` is missing `{needle}`: {joined}"
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
    inputs: &[VcfImputationFamilyAdapterArtifact],
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
            "VCF imputation-family adapter row `{stage_id}` unexpectedly accepted missing input `{}`",
            probe.artifact_id
        ),
        Err(error) => error.to_string(),
    };
    let passed = observed_error.contains(&expected_error_fragment);
    (probe.artifact_id, expected_error_fragment, observed_error, passed)
}

fn ensure_vcf_imputation_family_adapter_contract(
    registry_by_tool: &BTreeMap<String, RegistryToolContract>,
    rows: &[VcfImputationFamilyAdapterRow],
) -> Result<()> {
    let observed_tools = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>();
    let expected_tools = GOVERNED_IMPUTATION_TOOL_IDS.iter().copied().collect::<BTreeSet<_>>();
    if observed_tools != expected_tools {
        return Err(anyhow!(
            "VCF imputation-family adapter tool set drifted: expected {:?}, found {:?}",
            expected_tools,
            observed_tools
        ));
    }
    for tool_id in GOVERNED_IMPUTATION_TOOL_IDS {
        let registry_tool = registry_by_tool
            .get(tool_id)
            .ok_or_else(|| anyhow!("missing registry tool `{tool_id}`"))?;
        let tool_rows = rows.iter().filter(|row| row.tool_id == tool_id).collect::<Vec<_>>();
        let observed_stage_ids =
            tool_rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
        let expected_stage_ids =
            GOVERNED_IMPUTATION_STAGE_IDS.iter().copied().collect::<BTreeSet<_>>();
        if !registry_tool
            .stage_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            .is_superset(&expected_stage_ids)
        {
            return Err(anyhow!(
                "VCF registry row for `{tool_id}` does not cover the governed imputation stage slice"
            ));
        }
        if observed_stage_ids != expected_stage_ids {
            return Err(anyhow!(
                "VCF imputation-family adapter stage set drifted for `{tool_id}`: expected {:?}, found {:?}",
                expected_stage_ids,
                observed_stage_ids
            ));
        }
        for row in tool_rows {
            if row.tool_status != registry_tool.tool_status {
                return Err(anyhow!(
                    "VCF imputation-family row `{}` / `{tool_id}` drifted from registry tool status `{}`",
                    row.stage_id,
                    registry_tool.tool_status
                ));
            }
            if !row.argv_validation_passed {
                return Err(anyhow!(
                    "VCF imputation-family row `{}` / `{tool_id}` failed argv validation",
                    row.stage_id
                ));
            }
            if !row.missing_input_test_passed {
                return Err(anyhow!(
                    "VCF imputation-family row `{}` / `{tool_id}` failed missing-input validation: {}",
                    row.stage_id,
                    row.missing_input_observed_error
                ));
            }
            if row.parser_output_ids.is_empty() {
                return Err(anyhow!(
                    "VCF imputation-family row `{}` / `{tool_id}` is missing parser output declarations",
                    row.stage_id
                ));
            }
        }
    }
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    if benchmark_ready_row_count != 2 {
        return Err(anyhow!(
            "VCF imputation-family adapter benchmark-ready count drifted: expected 2, found {benchmark_ready_row_count}"
        ));
    }
    Ok(())
}

fn artifact(artifact_id: &str, role: &str, path: &str) -> VcfImputationFamilyAdapterArtifact {
    VcfImputationFamilyAdapterArtifact {
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
) -> VcfImputationFamilyAdapterCommandStep {
    VcfImputationFamilyAdapterCommandStep {
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

    use super::{render_vcf_imputation_family_adapter, DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("repo root")
    }

    #[test]
    fn vcf_imputation_family_adapter_tracks_governed_rows() {
        let repo_root = repo_root();
        let report = render_vcf_imputation_family_adapter(
            &repo_root,
            PathBuf::from(DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH),
        )
        .expect("render report");
        assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_imputation_family_adapter.v1");
        assert_eq!(report.row_count, 8);
        assert_eq!(report.tool_count, 4);
        assert_eq!(report.benchmark_ready_row_count, 2);
        assert_eq!(report.parser_output_row_count, 8);
        assert_eq!(report.missing_input_test_passed_row_count, 8);
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "minimac4"
                && row.stage_id == "vcf.impute"
                && row
                    .panel_m3vcf_path
                    .as_deref()
                    .is_some_and(|path| path.ends_with("panel.m3vcf.gz"))
        }));
    }
}
