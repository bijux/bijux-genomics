use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::taxonomy::VcfDomainStage;
use bijux_dna_stages_vcf::stage_specs::{
    vcf_domain_stage_adapter_id, vcf_domain_stage_expected_output_ids, vcf_domain_stage_parser_id,
};
use serde::Serialize;

use crate::commands::benchmark::local_vcf_stage_catalog::{
    build_vcf_stage_catalog_rows, VcfStageCatalogRow,
};
use crate::commands::benchmark::local_vcf_stage_matrix::{
    build_vcf_stage_matrix_rows, VcfStageMatrixRow,
};
use crate::commands::benchmark::readiness::vcf_readiness_inputs::load_governed_vcf_fixture_inputs;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_PLINK_ADAPTER_PATH: &str =
    "target/bench-readiness/adapters/plink.vcf.json";
pub(crate) const DEFAULT_VCF_PLINK2_ADAPTER_PATH: &str =
    "target/bench-readiness/adapters/plink2.vcf.json";
const VCF_PLINK_ADAPTER_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_plink_adapter.v1";
const VCF_PLINK2_ADAPTER_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_plink2_adapter.v1";
const GOVERNED_COHORT_VCF_PATH: &str =
    "tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfPlinkFamilyAdapterArtifact {
    pub(crate) artifact_id: String,
    pub(crate) role: String,
    pub(crate) path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfPlinkFamilyAdapterCommandStep {
    pub(crate) step_id: String,
    pub(crate) step_kind: String,
    pub(crate) argv: Vec<String>,
    pub(crate) declared_output_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfPlinkFamilyAdapterRow {
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
    pub(crate) normalized_metrics_artifact_id: String,
    pub(crate) normalized_metrics_path: String,
    pub(crate) stage_output_ids: Vec<String>,
    pub(crate) raw_output_ids: Vec<String>,
    pub(crate) parser_output_ids: Vec<String>,
    pub(crate) required_inputs: Vec<VcfPlinkFamilyAdapterArtifact>,
    pub(crate) declared_outputs: Vec<VcfPlinkFamilyAdapterArtifact>,
    pub(crate) command_steps: Vec<VcfPlinkFamilyAdapterCommandStep>,
    pub(crate) argv_validation_passed: bool,
    pub(crate) missing_input_probe_artifact_id: String,
    pub(crate) missing_input_expected_error_fragment: String,
    pub(crate) missing_input_observed_error: String,
    pub(crate) missing_input_test_passed: bool,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfPlinkFamilyAdapterReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) tool_id: String,
    pub(crate) tool_status: String,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) parser_output_row_count: usize,
    pub(crate) normalized_metrics_row_count: usize,
    pub(crate) raw_output_declared_row_count: usize,
    pub(crate) missing_input_test_passed_row_count: usize,
    pub(crate) rows: Vec<VcfPlinkFamilyAdapterRow>,
}

#[derive(Debug, Clone)]
struct RegistryToolContract {
    tool_id: String,
    tool_status: String,
    stage_ids: Vec<String>,
}

pub(crate) fn run_render_vcf_plink_adapter(
    args: &parse::BenchReadinessRenderVcfPlinkAdapterArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_plink_family_adapter(
        &repo_root,
        "plink",
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_PLINK_ADAPTER_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn run_render_vcf_plink2_adapter(
    args: &parse::BenchReadinessRenderVcfPlink2AdapterArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_plink_family_adapter(
        &repo_root,
        "plink2",
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_PLINK2_ADAPTER_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_plink_family_adapter(
    repo_root: &Path,
    tool_id: &str,
    output_path: PathBuf,
) -> Result<VcfPlinkFamilyAdapterReport> {
    let registry_tool = load_registry_tool_contract(repo_root, tool_id)?;
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_vcf_plink_family_adapter_rows(repo_root, &registry_tool)?;
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let parser_output_row_count =
        rows.iter().filter(|row| !row.parser_output_ids.is_empty()).count();
    let normalized_metrics_row_count =
        rows.iter().filter(|row| !row.normalized_metrics_path.is_empty()).count();
    let raw_output_declared_row_count =
        rows.iter().filter(|row| !row.raw_output_ids.is_empty()).count();
    let missing_input_test_passed_row_count =
        rows.iter().filter(|row| row.missing_input_test_passed).count();

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let report = VcfPlinkFamilyAdapterReport {
        schema_version: schema_version_for_tool(tool_id),
        domain: "vcf",
        tool_id: registry_tool.tool_id.clone(),
        tool_status: registry_tool.tool_status.clone(),
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        benchmark_ready_row_count,
        parser_output_row_count,
        normalized_metrics_row_count,
        raw_output_declared_row_count,
        missing_input_test_passed_row_count,
        rows,
    };
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    Ok(report)
}

fn collect_vcf_plink_family_adapter_rows(
    repo_root: &Path,
    registry_tool: &RegistryToolContract,
) -> Result<Vec<VcfPlinkFamilyAdapterRow>> {
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
        rows.push(build_plink_family_row(
            repo_root,
            registry_tool,
            stage_id,
            catalog_row,
            matrix_by_stage.get(stage_id.as_str()),
        )?);
    }
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    ensure_vcf_plink_family_adapter_contract(registry_tool, &rows)?;
    Ok(rows)
}

fn build_plink_family_row(
    repo_root: &Path,
    registry_tool: &RegistryToolContract,
    stage_id: &str,
    catalog_row: &VcfStageCatalogRow,
    matrix_row: Option<&VcfStageMatrixRow>,
) -> Result<VcfPlinkFamilyAdapterRow> {
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
        .unwrap_or_else(|| "vcf_cohort".to_string());
    let output_root =
        format!("target/bench-readiness/adapters/{}/{}", registry_tool.tool_id, stage_id);
    let output_prefix = format!("{output_root}/{}", stage_output_name_hint(stage));
    let fixture_inputs = load_governed_vcf_fixture_inputs(repo_root)?;
    let required_inputs = governed_inputs_for_stage(stage, &fixture_inputs.sample_metadata_path);
    validate_required_inputs(repo_root, stage_id, &required_inputs)?;
    let stage_output_ids = vcf_domain_stage_expected_output_ids(stage)
        .ok_or_else(|| anyhow!("VCF {stage_id} is missing expected output ids"))?
        .iter()
        .map(|output| (*output).to_string())
        .collect::<Vec<_>>();
    let (
        command_contract_source,
        normalized_metrics_artifact_id,
        normalized_metrics_path,
        raw_output_ids,
        parser_output_ids,
        declared_outputs,
        command_steps,
    ) = build_stage_contract(registry_tool.tool_id.as_str(), stage, &output_root, &output_prefix)?;
    let argv_validation_passed =
        validate_command_steps(registry_tool.tool_id.as_str(), stage_id, &command_steps).is_ok();
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
    let reason = match (registry_tool.tool_id.as_str(), stage) {
        ("plink2", VcfDomainStage::Admixture) => format!(
            "row `{stage_id}` / `{}` keeps the PLINK2 PCA-proxy raw outputs explicit and maps them to `{}` instead of pretending PLINK2 owns a native Q-matrix",
            registry_tool.tool_id, normalized_metrics_artifact_id
        ),
        ("plink", VcfDomainStage::Admixture) => format!(
            "row `{stage_id}` / `{}` keeps the PLINK cohort-preparation outputs explicit and maps them to `{}` for the downstream admixture report contract",
            registry_tool.tool_id, normalized_metrics_artifact_id
        ),
        _ => format!(
            "row `{stage_id}` / `{}` renders concrete cohort-analysis argv with {} declared raw output(s) and maps them to `{}`",
            registry_tool.tool_id,
            raw_output_ids.len(),
            normalized_metrics_artifact_id
        ),
    };

    Ok(VcfPlinkFamilyAdapterRow {
        tool_id: registry_tool.tool_id.clone(),
        tool_status: registry_tool.tool_status.clone(),
        stage_id: stage_id.to_string(),
        stage_status: catalog_row.support_status.clone(),
        benchmark_status,
        adapter_id,
        parser_id,
        corpus_id,
        asset_profile_id,
        command_contract_source,
        output_root,
        output_prefix,
        normalized_metrics_artifact_id,
        normalized_metrics_path,
        stage_output_ids,
        raw_output_ids,
        parser_output_ids,
        required_inputs,
        declared_outputs,
        command_steps,
        argv_validation_passed,
        missing_input_probe_artifact_id,
        missing_input_expected_error_fragment,
        missing_input_observed_error,
        missing_input_test_passed,
        reason,
    })
}

fn governed_inputs_for_stage(
    stage: VcfDomainStage,
    sample_metadata_path: &str,
) -> Vec<VcfPlinkFamilyAdapterArtifact> {
    let mut inputs = vec![artifact("vcf", "variant", GOVERNED_COHORT_VCF_PATH)];
    if matches!(
        stage,
        VcfDomainStage::Pca | VcfDomainStage::Admixture | VcfDomainStage::PopulationStructure
    ) {
        inputs.push(artifact("sample_metadata_manifest", "sample_metadata", sample_metadata_path));
    }
    inputs
}

#[allow(clippy::type_complexity)]
fn build_stage_contract(
    tool_id: &str,
    stage: VcfDomainStage,
    output_root: &str,
    output_prefix: &str,
) -> Result<(
    String,
    String,
    String,
    Vec<String>,
    Vec<String>,
    Vec<VcfPlinkFamilyAdapterArtifact>,
    Vec<VcfPlinkFamilyAdapterCommandStep>,
)> {
    let json_output_path = |name: &str| format!("{output_root}/{name}.json");
    let raw_path = |suffix: &str| format!("{output_prefix}.{suffix}");
    let contract = match (tool_id, stage) {
        ("plink", VcfDomainStage::Qc) => (
            "domain/vcf/fixtures/vcf.qc/plink.txt".to_string(),
            "qc_report".to_string(),
            json_output_path("qc_report"),
            vec![
                "sample_missingness_imiss".to_string(),
                "variant_missingness_lmiss".to_string(),
                "allele_frequency_frq".to_string(),
                "heterozygosity_het".to_string(),
                "hardy_weinberg_hwe".to_string(),
                "plink_log".to_string(),
            ],
            vec!["qc_report".to_string()],
            vec![
                artifact("sample_missingness_imiss", "report_tsv", &raw_path("imiss")),
                artifact("variant_missingness_lmiss", "report_tsv", &raw_path("lmiss")),
                artifact("allele_frequency_frq", "report_tsv", &raw_path("frq")),
                artifact("heterozygosity_het", "report_tsv", &raw_path("het")),
                artifact("hardy_weinberg_hwe", "report_tsv", &raw_path("hwe")),
                artifact("plink_log", "log", &raw_path("log")),
                artifact("qc_report", "report_json", &json_output_path("qc_report")),
            ],
            vec![step(
                "qc",
                "quality_control",
                vec![
                    "plink",
                    "--vcf",
                    GOVERNED_COHORT_VCF_PATH,
                    "--double-id",
                    "--allow-extra-chr",
                    "--missing",
                    "--freq",
                    "--het",
                    "--hardy",
                    "--out",
                    output_prefix,
                ],
                &[
                    "sample_missingness_imiss",
                    "variant_missingness_lmiss",
                    "allele_frequency_frq",
                    "heterozygosity_het",
                    "hardy_weinberg_hwe",
                    "plink_log",
                ],
            )],
        ),
        ("plink", VcfDomainStage::Admixture) => (
            "domain/vcf/fixtures/vcf.admixture/plink.txt".to_string(),
            "admixture_report".to_string(),
            json_output_path("admixture_report"),
            vec![
                "bed_matrix".to_string(),
                "bim_index".to_string(),
                "fam_samples".to_string(),
                "allele_frequency_frq".to_string(),
                "sample_missingness_imiss".to_string(),
                "variant_missingness_lmiss".to_string(),
                "plink_log".to_string(),
            ],
            vec!["admixture_report".to_string()],
            vec![
                artifact("bed_matrix", "bed", &raw_path("bed")),
                artifact("bim_index", "bim", &raw_path("bim")),
                artifact("fam_samples", "fam", &raw_path("fam")),
                artifact("allele_frequency_frq", "report_tsv", &raw_path("frq")),
                artifact("sample_missingness_imiss", "report_tsv", &raw_path("imiss")),
                artifact("variant_missingness_lmiss", "report_tsv", &raw_path("lmiss")),
                artifact("plink_log", "log", &raw_path("log")),
                artifact("admixture_report", "report_json", &json_output_path("admixture_report")),
            ],
            vec![step(
                "admixture_prep",
                "cohort_prep",
                vec![
                    "plink",
                    "--vcf",
                    GOVERNED_COHORT_VCF_PATH,
                    "--double-id",
                    "--allow-extra-chr",
                    "--missing",
                    "--freq",
                    "--make-bed",
                    "--out",
                    output_prefix,
                ],
                &[
                    "bed_matrix",
                    "bim_index",
                    "fam_samples",
                    "allele_frequency_frq",
                    "sample_missingness_imiss",
                    "variant_missingness_lmiss",
                    "plink_log",
                ],
            )],
        ),
        ("plink2", VcfDomainStage::Qc) => (
            "domain/vcf/fixtures/vcf.qc/plink2.txt".to_string(),
            "qc_report".to_string(),
            json_output_path("qc_report"),
            vec![
                "sample_missingness_smiss".to_string(),
                "variant_missingness_vmiss".to_string(),
                "allele_frequency_afreq".to_string(),
                "heterozygosity_het".to_string(),
                "hardy_weinberg_hardy".to_string(),
                "plink2_log".to_string(),
            ],
            vec!["qc_report".to_string()],
            vec![
                artifact("sample_missingness_smiss", "report_tsv", &raw_path("smiss")),
                artifact("variant_missingness_vmiss", "report_tsv", &raw_path("vmiss")),
                artifact("allele_frequency_afreq", "report_tsv", &raw_path("afreq")),
                artifact("heterozygosity_het", "report_tsv", &raw_path("het")),
                artifact("hardy_weinberg_hardy", "report_tsv", &raw_path("hardy")),
                artifact("plink2_log", "log", &raw_path("log")),
                artifact("qc_report", "report_json", &json_output_path("qc_report")),
            ],
            vec![step(
                "qc",
                "quality_control",
                vec![
                    "plink2",
                    "--vcf",
                    GOVERNED_COHORT_VCF_PATH,
                    "--double-id",
                    "--allow-extra-chr",
                    "--missing",
                    "--freq",
                    "--het",
                    "--hardy",
                    "--out",
                    output_prefix,
                ],
                &[
                    "sample_missingness_smiss",
                    "variant_missingness_vmiss",
                    "allele_frequency_afreq",
                    "heterozygosity_het",
                    "hardy_weinberg_hardy",
                    "plink2_log",
                ],
            )],
        ),
        ("plink2", VcfDomainStage::Pca) => (
            "domain/vcf/fixtures/vcf.pca/plink2.txt".to_string(),
            "pca_report".to_string(),
            json_output_path("pca_report"),
            vec!["pca_eigenvec".to_string(), "pca_eigenval".to_string(), "plink2_log".to_string()],
            vec!["pca_report".to_string()],
            vec![
                artifact("pca_eigenvec", "report_tsv", &raw_path("eigenvec")),
                artifact("pca_eigenval", "report_tsv", &raw_path("eigenval")),
                artifact("plink2_log", "log", &raw_path("log")),
                artifact("pca_report", "report_json", &json_output_path("pca_report")),
            ],
            vec![step(
                "pca",
                "principal_components",
                vec![
                    "plink2",
                    "--vcf",
                    GOVERNED_COHORT_VCF_PATH,
                    "--double-id",
                    "--allow-extra-chr",
                    "--pca",
                    "10",
                    "--out",
                    output_prefix,
                ],
                &["pca_eigenvec", "pca_eigenval", "plink2_log"],
            )],
        ),
        ("plink2", VcfDomainStage::Admixture) => (
            "domain/vcf/fixtures/vcf.admixture/plink2.txt".to_string(),
            "admixture_report".to_string(),
            json_output_path("admixture_report"),
            vec![
                "admixture_proxy_eigenvec".to_string(),
                "admixture_proxy_eigenval".to_string(),
                "plink2_log".to_string(),
            ],
            vec!["admixture_report".to_string()],
            vec![
                artifact("admixture_proxy_eigenvec", "report_tsv", &raw_path("eigenvec")),
                artifact("admixture_proxy_eigenval", "report_tsv", &raw_path("eigenval")),
                artifact("plink2_log", "log", &raw_path("log")),
                artifact("admixture_report", "report_json", &json_output_path("admixture_report")),
            ],
            vec![step(
                "admixture_proxy",
                "population_proxy",
                vec![
                    "plink2",
                    "--vcf",
                    GOVERNED_COHORT_VCF_PATH,
                    "--double-id",
                    "--allow-extra-chr",
                    "--pca",
                    "2",
                    "--out",
                    output_prefix,
                ],
                &["admixture_proxy_eigenvec", "admixture_proxy_eigenval", "plink2_log"],
            )],
        ),
        ("plink2", VcfDomainStage::PopulationStructure) => {
            let prune_prefix = format!("{output_prefix}.prune");
            let pca_prefix = format!("{output_prefix}.pca");
            (
                "domain/vcf/fixtures/vcf.population_structure/plink2.txt".to_string(),
                "population_structure_report".to_string(),
                json_output_path("population_structure_report"),
                vec![
                    "ld_prune_in".to_string(),
                    "ld_prune_out".to_string(),
                    "population_pca_eigenvec".to_string(),
                    "population_pca_eigenval".to_string(),
                    "prune_log".to_string(),
                    "pca_log".to_string(),
                ],
                vec!["population_structure_report".to_string()],
                vec![
                    artifact("ld_prune_in", "report_tsv", &format!("{prune_prefix}.prune.in")),
                    artifact("ld_prune_out", "report_tsv", &format!("{prune_prefix}.prune.out")),
                    artifact(
                        "population_pca_eigenvec",
                        "report_tsv",
                        &format!("{pca_prefix}.eigenvec"),
                    ),
                    artifact(
                        "population_pca_eigenval",
                        "report_tsv",
                        &format!("{pca_prefix}.eigenval"),
                    ),
                    artifact("prune_log", "log", &format!("{prune_prefix}.log")),
                    artifact("pca_log", "log", &format!("{pca_prefix}.log")),
                    artifact(
                        "population_structure_report",
                        "report_json",
                        &json_output_path("population_structure_report"),
                    ),
                ],
                vec![
                    step(
                        "ld_prune",
                        "ld_pruning",
                        vec![
                            "plink2",
                            "--vcf",
                            GOVERNED_COHORT_VCF_PATH,
                            "--double-id",
                            "--allow-extra-chr",
                            "--indep-pairwise",
                            "50",
                            "5",
                            "0.2",
                            "--out",
                            &prune_prefix,
                        ],
                        &["ld_prune_in", "ld_prune_out", "prune_log"],
                    ),
                    step(
                        "population_pca",
                        "principal_components",
                        vec![
                            "plink2",
                            "--vcf",
                            GOVERNED_COHORT_VCF_PATH,
                            "--double-id",
                            "--allow-extra-chr",
                            "--pca",
                            "10",
                            "--out",
                            &pca_prefix,
                        ],
                        &["population_pca_eigenvec", "population_pca_eigenval", "pca_log"],
                    ),
                ],
            )
        }
        ("plink2", VcfDomainStage::Roh) => (
            "domain/vcf/fixtures/vcf.roh/plink2.txt".to_string(),
            "roh_report".to_string(),
            json_output_path("roh_report"),
            vec!["roh_hom".to_string(), "plink2_log".to_string()],
            vec!["roh_report".to_string()],
            vec![
                artifact("roh_hom", "report_tsv", &raw_path("hom")),
                artifact("plink2_log", "log", &raw_path("log")),
                artifact("roh_report", "report_json", &json_output_path("roh_report")),
            ],
            vec![step(
                "roh",
                "runs_of_homozygosity",
                vec![
                    "plink2",
                    "--vcf",
                    GOVERNED_COHORT_VCF_PATH,
                    "--double-id",
                    "--allow-extra-chr",
                    "--homozyg",
                    "--out",
                    output_prefix,
                ],
                &["roh_hom", "plink2_log"],
            )],
        ),
        _ => {
            return Err(anyhow!(
                "VCF {tool_id} adapter does not govern stage `{}`",
                stage.as_str()
            ));
        }
    };
    Ok(contract)
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
        "plink" => VCF_PLINK_ADAPTER_SCHEMA_VERSION,
        "plink2" => VCF_PLINK2_ADAPTER_SCHEMA_VERSION,
        other => panic!("unsupported VCF plink family schema tool `{other}`"),
    }
}

fn stage_output_name_hint(stage: VcfDomainStage) -> &'static str {
    match stage {
        VcfDomainStage::Qc => "qc",
        VcfDomainStage::Pca => "pca",
        VcfDomainStage::Admixture => "admixture",
        VcfDomainStage::PopulationStructure => "population_structure",
        VcfDomainStage::Roh => "roh",
        other => panic!("unsupported VCF plink family stage `{}`", other.as_str()),
    }
}

fn validate_required_inputs(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfPlinkFamilyAdapterArtifact],
) -> Result<()> {
    for input in inputs {
        let resolved = repo_root.join(&input.path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "VCF plink-family adapter for `{stage_id}` is missing required input `{}` at `{}`",
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
    steps: &[VcfPlinkFamilyAdapterCommandStep],
) -> Result<()> {
    if steps.is_empty() {
        return Err(anyhow!(
            "VCF {tool_id} adapter row `{stage_id}` must declare at least one command step"
        ));
    }
    for step in steps {
        if step.argv.is_empty() {
            return Err(anyhow!(
                "VCF {tool_id} adapter step `{}` for `{stage_id}` has empty argv",
                step.step_id
            ));
        }
        if step.argv.iter().any(|part| {
            let lowered = part.to_ascii_lowercase();
            lowered.contains("placeholder") || lowered == "--help" || lowered.contains("todo")
        }) {
            return Err(anyhow!(
                "VCF {tool_id} adapter step `{}` for `{stage_id}` still contains placeholder argv: {:?}",
                step.step_id,
                step.argv
            ));
        }
        if !step.argv.iter().any(|part| part == "--out") {
            return Err(anyhow!(
                "VCF {tool_id} adapter step `{}` for `{stage_id}` is missing `--out`",
                step.step_id
            ));
        }
    }
    let joined =
        steps.iter().flat_map(|step| step.argv.iter().cloned()).collect::<Vec<_>>().join(" ");
    match (tool_id, stage_id) {
        ("plink", "vcf.qc") => {
            ensure_join_contains(stage_id, &joined, "--missing")?;
            ensure_join_contains(stage_id, &joined, "--freq")?;
            ensure_join_contains(stage_id, &joined, "--het")?;
            ensure_join_contains(stage_id, &joined, "--hardy")?;
        }
        ("plink", "vcf.admixture") => {
            ensure_join_contains(stage_id, &joined, "--make-bed")?;
            ensure_join_contains(stage_id, &joined, "--freq")?;
            ensure_join_contains(stage_id, &joined, "--missing")?;
        }
        ("plink2", "vcf.qc") => {
            ensure_join_contains(stage_id, &joined, "--missing")?;
            ensure_join_contains(stage_id, &joined, "--freq")?;
            ensure_join_contains(stage_id, &joined, "--het")?;
            ensure_join_contains(stage_id, &joined, "--hardy")?;
        }
        ("plink2", "vcf.pca") => ensure_join_contains(stage_id, &joined, "--pca")?,
        ("plink2", "vcf.admixture") => {
            ensure_join_contains(stage_id, &joined, "--pca")?;
            ensure_join_contains(stage_id, &joined, " 2 ")?;
        }
        ("plink2", "vcf.population_structure") => {
            ensure_join_contains(stage_id, &joined, "--indep-pairwise")?;
            ensure_join_contains(stage_id, &joined, "--pca")?;
        }
        ("plink2", "vcf.roh") => ensure_join_contains(stage_id, &joined, "--homozyg")?,
        _ => {}
    }
    Ok(())
}

fn ensure_join_contains(stage_id: &str, joined: &str, needle: &str) -> Result<()> {
    if joined.contains(needle) {
        Ok(())
    } else {
        Err(anyhow!(
            "VCF plink-family adapter row `{stage_id}` is missing required argv token `{needle}`: {joined}"
        ))
    }
}

fn run_missing_input_probe(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfPlinkFamilyAdapterArtifact],
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
            "VCF plink-family adapter row `{stage_id}` unexpectedly accepted missing input `{}`",
            probe.artifact_id
        ),
        Err(error) => error.to_string(),
    };
    let passed = observed_error.contains(&expected_error_fragment);
    (probe.artifact_id, expected_error_fragment, observed_error, passed)
}

fn ensure_vcf_plink_family_adapter_contract(
    registry_tool: &RegistryToolContract,
    rows: &[VcfPlinkFamilyAdapterRow],
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
        if row.normalized_metrics_path.is_empty() {
            return Err(anyhow!(
                "VCF {} adapter row `{}` is missing normalized metrics mapping",
                registry_tool.tool_id,
                row.stage_id
            ));
        }
    }
    Ok(())
}

fn artifact(artifact_id: &str, role: &str, path: &str) -> VcfPlinkFamilyAdapterArtifact {
    VcfPlinkFamilyAdapterArtifact {
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
) -> VcfPlinkFamilyAdapterCommandStep {
    VcfPlinkFamilyAdapterCommandStep {
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
        render_vcf_plink_family_adapter, DEFAULT_VCF_PLINK2_ADAPTER_PATH,
        DEFAULT_VCF_PLINK_ADAPTER_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_plink_adapter_tracks_governed_rows() {
        let repo_root = repo_root();
        let report = render_vcf_plink_family_adapter(
            &repo_root,
            "plink",
            PathBuf::from(DEFAULT_VCF_PLINK_ADAPTER_PATH),
        )
        .expect("render VCF plink adapter");

        assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_plink_adapter.v1");
        assert_eq!(report.tool_id, "plink");
        assert_eq!(report.tool_status, "experimental");
        assert_eq!(report.row_count, 2);
        assert_eq!(report.benchmark_ready_row_count, 0);
        assert_eq!(report.parser_output_row_count, 2);
        assert_eq!(report.normalized_metrics_row_count, 2);
        assert_eq!(report.raw_output_declared_row_count, 2);

        let qc = report.rows.iter().find(|row| row.stage_id == "vcf.qc").expect("plink qc row");
        assert_eq!(qc.normalized_metrics_artifact_id, "qc_report");
        assert!(
            qc.command_steps[0].argv.iter().any(|part| part == "--missing"),
            "plink qc row must retain missingness output flags"
        );
        let admixture = report
            .rows
            .iter()
            .find(|row| row.stage_id == "vcf.admixture")
            .expect("plink admixture row");
        assert!(
            admixture.command_steps[0].argv.iter().any(|part| part == "--make-bed"),
            "plink admixture row must retain cohort-preparation outputs"
        );
    }

    #[test]
    fn vcf_plink2_adapter_tracks_governed_rows() {
        let repo_root = repo_root();
        let report = render_vcf_plink_family_adapter(
            &repo_root,
            "plink2",
            PathBuf::from(DEFAULT_VCF_PLINK2_ADAPTER_PATH),
        )
        .expect("render VCF plink2 adapter");

        assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_plink2_adapter.v1");
        assert_eq!(report.tool_id, "plink2");
        assert_eq!(report.tool_status, "experimental");
        assert_eq!(report.row_count, 5);
        assert_eq!(report.benchmark_ready_row_count, 5);
        assert_eq!(report.parser_output_row_count, 5);
        assert_eq!(report.normalized_metrics_row_count, 5);
        assert_eq!(report.raw_output_declared_row_count, 5);

        let pca = report.rows.iter().find(|row| row.stage_id == "vcf.pca").expect("plink2 pca row");
        assert_eq!(pca.normalized_metrics_artifact_id, "pca_report");
        assert!(
            pca.command_steps[0].argv.iter().any(|part| part == "--pca"),
            "plink2 pca row must keep eigenvector/eigenvalue output flags"
        );
        let roh = report.rows.iter().find(|row| row.stage_id == "vcf.roh").expect("plink2 roh row");
        assert!(
            roh.command_steps[0].argv.iter().any(|part| part == "--homozyg"),
            "plink2 roh row must retain ROH HOM output flags"
        );
        let population_structure = report
            .rows
            .iter()
            .find(|row| row.stage_id == "vcf.population_structure")
            .expect("plink2 population_structure row");
        assert_eq!(population_structure.command_steps.len(), 2);
    }
}
