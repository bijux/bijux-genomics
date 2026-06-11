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
use crate::commands::benchmark::local_vcf_stage_matrix::VcfStageMatrixRow;
use crate::commands::benchmark::vcf_benchmark_bindings::collect_vcf_benchmark_binding_rows;
use crate::commands::benchmark::readiness::vcf_readiness_inputs::load_governed_vcf_fixture_inputs;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH: &str =
    "benchmarks/readiness/adapters/eigensoft.vcf.json";
const VCF_EIGENSOFT_ADAPTER_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_eigensoft_adapter.v1";
const GOVERNED_COHORT_VCF_PATH: &str =
    "benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfEigensoftAdapterArtifact {
    pub(crate) artifact_id: String,
    pub(crate) role: String,
    pub(crate) path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfEigensoftAdapterCommandStep {
    pub(crate) step_id: String,
    pub(crate) step_kind: String,
    pub(crate) argv: Vec<String>,
    pub(crate) declared_output_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfEigensoftAdapterRow {
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
    pub(crate) convertf_par_path: String,
    pub(crate) smartpca_par_path: String,
    pub(crate) geno_path: String,
    pub(crate) snp_path: String,
    pub(crate) ind_path: String,
    pub(crate) eigenvec_path: String,
    pub(crate) eigenval_path: String,
    pub(crate) smartpca_log_path: String,
    pub(crate) normalized_metrics_artifact_id: String,
    pub(crate) normalized_metrics_path: String,
    pub(crate) stage_output_ids: Vec<String>,
    pub(crate) raw_output_ids: Vec<String>,
    pub(crate) parser_output_ids: Vec<String>,
    pub(crate) required_inputs: Vec<VcfEigensoftAdapterArtifact>,
    pub(crate) declared_outputs: Vec<VcfEigensoftAdapterArtifact>,
    pub(crate) command_steps: Vec<VcfEigensoftAdapterCommandStep>,
    pub(crate) argv_validation_passed: bool,
    pub(crate) missing_input_probe_artifact_id: String,
    pub(crate) missing_input_expected_error_fragment: String,
    pub(crate) missing_input_observed_error: String,
    pub(crate) missing_input_test_passed: bool,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfEigensoftAdapterReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) tool_id: String,
    pub(crate) tool_status: String,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) parser_output_row_count: usize,
    pub(crate) normalized_metrics_row_count: usize,
    pub(crate) conversion_output_row_count: usize,
    pub(crate) pca_output_row_count: usize,
    pub(crate) missing_input_test_passed_row_count: usize,
    pub(crate) rows: Vec<VcfEigensoftAdapterRow>,
}

#[derive(Debug, Clone)]
struct RegistryToolContract {
    tool_id: String,
    tool_status: String,
    stage_ids: Vec<String>,
}

struct StageContract {
    command_contract_source: String,
    convertf_par_path: String,
    smartpca_par_path: String,
    geno_path: String,
    snp_path: String,
    ind_path: String,
    eigenvec_path: String,
    eigenval_path: String,
    smartpca_log_path: String,
    normalized_metrics_artifact_id: String,
    normalized_metrics_path: String,
    raw_output_ids: Vec<String>,
    parser_output_ids: Vec<String>,
    declared_outputs: Vec<VcfEigensoftAdapterArtifact>,
    command_steps: Vec<VcfEigensoftAdapterCommandStep>,
}

pub(crate) fn run_render_vcf_eigensoft_adapter(
    args: &parse::BenchReadinessRenderVcfEigensoftAdapterArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_eigensoft_adapter(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_eigensoft_adapter(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfEigensoftAdapterReport> {
    let registry_tool = load_registry_tool_contract(repo_root, "eigensoft")?;
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_vcf_eigensoft_adapter_rows(repo_root, &registry_tool)?;
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let parser_output_row_count =
        rows.iter().filter(|row| !row.parser_output_ids.is_empty()).count();
    let normalized_metrics_row_count =
        rows.iter().filter(|row| !row.normalized_metrics_path.is_empty()).count();
    let conversion_output_row_count = rows
        .iter()
        .filter(|row| {
            !row.geno_path.is_empty() && !row.snp_path.is_empty() && !row.ind_path.is_empty()
        })
        .count();
    let pca_output_row_count = rows
        .iter()
        .filter(|row| !row.eigenvec_path.is_empty() && !row.eigenval_path.is_empty())
        .count();
    let missing_input_test_passed_row_count =
        rows.iter().filter(|row| row.missing_input_test_passed).count();

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let report = VcfEigensoftAdapterReport {
        schema_version: VCF_EIGENSOFT_ADAPTER_SCHEMA_VERSION,
        domain: "vcf",
        tool_id: registry_tool.tool_id.clone(),
        tool_status: registry_tool.tool_status.clone(),
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        benchmark_ready_row_count,
        parser_output_row_count,
        normalized_metrics_row_count,
        conversion_output_row_count,
        pca_output_row_count,
        missing_input_test_passed_row_count,
        rows,
    };
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    Ok(report)
}

fn collect_vcf_eigensoft_adapter_rows(
    repo_root: &Path,
    registry_tool: &RegistryToolContract,
) -> Result<Vec<VcfEigensoftAdapterRow>> {
    let catalog_by_stage = build_vcf_stage_catalog_rows()?
        .into_iter()
        .map(|row| (row.stage_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let matrix_by_stage = collect_vcf_benchmark_binding_rows()?
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
        rows.push(build_eigensoft_row(
            repo_root,
            registry_tool,
            stage_id,
            catalog_row,
            matrix_by_stage.get(stage_id.as_str()),
        )?);
    }
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    ensure_vcf_eigensoft_adapter_contract(registry_tool, &rows)?;
    Ok(rows)
}

pub(crate) fn collect_vcf_eigensoft_adapter_rows_for_tool(
    repo_root: &Path,
) -> Result<Vec<VcfEigensoftAdapterRow>> {
    let registry_tool = load_registry_tool_contract(repo_root, "eigensoft")?;
    collect_vcf_eigensoft_adapter_rows(repo_root, &registry_tool)
}

fn build_eigensoft_row(
    repo_root: &Path,
    registry_tool: &RegistryToolContract,
    stage_id: &str,
    catalog_row: &VcfStageCatalogRow,
    matrix_row: Option<&VcfStageMatrixRow>,
) -> Result<VcfEigensoftAdapterRow> {
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
        format!("benchmarks/readiness/adapters/{}/{}", registry_tool.tool_id, stage_id);
    let output_prefix = format!("{output_root}/{}", stage_output_name_hint(stage));
    let fixture_inputs = load_governed_vcf_fixture_inputs(repo_root)?;
    let required_inputs = vec![
        artifact("vcf", "variant", GOVERNED_COHORT_VCF_PATH),
        artifact(
            "sample_metadata_manifest",
            "sample_metadata",
            &fixture_inputs.sample_metadata_path,
        ),
    ];
    validate_required_inputs(repo_root, stage_id, &required_inputs)?;
    let stage_output_ids = vcf_domain_stage_expected_output_ids(stage)
        .ok_or_else(|| anyhow!("VCF {stage_id} is missing expected output ids"))?
        .iter()
        .map(|output| (*output).to_string())
        .collect::<Vec<_>>();
    let contract = build_stage_contract(stage, &output_root, &output_prefix)?;
    let argv_validation_passed = validate_command_steps(stage_id, &contract.command_steps).is_ok();
    let (
        missing_input_probe_artifact_id,
        missing_input_expected_error_fragment,
        missing_input_observed_error,
        missing_input_test_passed,
    ) = run_missing_input_probe(repo_root, stage_id, &required_inputs);

    let reason = format!(
        "row `{stage_id}` / `{}` renders explicit convertf inputs `{}`, `{}`, `{}` and smartpca outputs `{}` / `{}` before mapping them to `{}`",
        registry_tool.tool_id,
        contract.geno_path,
        contract.snp_path,
        contract.ind_path,
        contract.eigenvec_path,
        contract.eigenval_path,
        contract.normalized_metrics_artifact_id
    );

    Ok(VcfEigensoftAdapterRow {
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
        convertf_par_path: contract.convertf_par_path,
        smartpca_par_path: contract.smartpca_par_path,
        geno_path: contract.geno_path,
        snp_path: contract.snp_path,
        ind_path: contract.ind_path,
        eigenvec_path: contract.eigenvec_path,
        eigenval_path: contract.eigenval_path,
        smartpca_log_path: contract.smartpca_log_path,
        normalized_metrics_artifact_id: contract.normalized_metrics_artifact_id,
        normalized_metrics_path: contract.normalized_metrics_path,
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
    stage: VcfDomainStage,
    output_root: &str,
    output_prefix: &str,
) -> Result<StageContract> {
    let convertf_par_path = format!("{output_prefix}.convertf.par");
    let smartpca_par_path = format!("{output_prefix}.smartpca.par");
    let geno_path = format!("{output_prefix}.geno");
    let snp_path = format!("{output_prefix}.snp");
    let ind_path = format!("{output_prefix}.ind");
    let eigenvec_path = format!("{output_prefix}.evec");
    let eigenval_path = format!("{output_prefix}.eval");
    let smartpca_log_path = format!("{output_prefix}.smartpca.log");
    let (command_contract_source, normalized_metrics_artifact_id) = match stage {
        VcfDomainStage::Pca => ("domain/vcf/fixtures/vcf.pca/eigensoft.txt", "pca_report"),
        VcfDomainStage::PopulationStructure => (
            "domain/vcf/fixtures/vcf.population_structure/eigensoft.txt",
            "population_structure_report",
        ),
        other => {
            return Err(anyhow!(
                "VCF eigensoft adapter does not govern stage `{}`",
                other.as_str()
            ));
        }
    };
    let normalized_metrics_path = format!("{output_root}/{normalized_metrics_artifact_id}.json");
    let raw_output_ids = vec![
        "convertf_par".to_string(),
        "smartpca_par".to_string(),
        "eigensoft_geno".to_string(),
        "eigensoft_snp".to_string(),
        "eigensoft_ind".to_string(),
        "smartpca_eigenvec".to_string(),
        "smartpca_eigenval".to_string(),
        "smartpca_log".to_string(),
    ];
    let parser_output_ids = vec![normalized_metrics_artifact_id.to_string()];
    let declared_outputs = vec![
        artifact("convertf_par", "config", &convertf_par_path),
        artifact("smartpca_par", "config", &smartpca_par_path),
        artifact("eigensoft_geno", "variant_matrix", &geno_path),
        artifact("eigensoft_snp", "variant_index", &snp_path),
        artifact("eigensoft_ind", "sample_manifest", &ind_path),
        artifact("smartpca_eigenvec", "report_tsv", &eigenvec_path),
        artifact("smartpca_eigenval", "report_tsv", &eigenval_path),
        artifact("smartpca_log", "log", &smartpca_log_path),
        artifact(normalized_metrics_artifact_id, "report_json", &normalized_metrics_path),
    ];
    let command_steps = vec![
        step(
            "write_convertf_par",
            "config_render",
            vec![
                "sh",
                "-lc",
                &format!(
                    "cat > '{convertf_par_path}' <<'EOF'\n\
genotypename: {GOVERNED_COHORT_VCF_PATH}\n\
snpname: {snp_path}\n\
indivname: {ind_path}\n\
outputformat: EIGENSTRAT\n\
genotypeoutname: {geno_path}\n\
snpoutname: {snp_path}\n\
indivoutname: {ind_path}\n\
familynames: NO\n\
EOF"
                ),
            ],
            &["convertf_par"],
        ),
        step(
            "convertf",
            "format_conversion",
            vec!["convertf", "-p", &convertf_par_path],
            &["eigensoft_geno", "eigensoft_snp", "eigensoft_ind"],
        ),
        step(
            "write_smartpca_par",
            "config_render",
            vec![
                "sh",
                "-lc",
                &format!(
                    "cat > '{smartpca_par_path}' <<'EOF'\n\
genotypename: {geno_path}\n\
snpname: {snp_path}\n\
indivname: {ind_path}\n\
evecoutname: {eigenvec_path}\n\
evaloutname: {eigenval_path}\n\
numoutevec: 10\n\
familynames: NO\n\
EOF"
                ),
            ],
            &["smartpca_par"],
        ),
        step(
            "smartpca",
            "principal_components",
            vec![
                "sh",
                "-lc",
                &format!("smartpca -p '{smartpca_par_path}' > '{smartpca_log_path}' 2>&1"),
            ],
            &["smartpca_eigenvec", "smartpca_eigenval", "smartpca_log"],
        ),
    ];

    Ok(StageContract {
        command_contract_source: command_contract_source.to_string(),
        convertf_par_path,
        smartpca_par_path,
        geno_path,
        snp_path,
        ind_path,
        eigenvec_path,
        eigenval_path,
        smartpca_log_path,
        normalized_metrics_artifact_id: normalized_metrics_artifact_id.to_string(),
        normalized_metrics_path,
        raw_output_ids,
        parser_output_ids,
        declared_outputs,
        command_steps,
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

fn stage_output_name_hint(stage: VcfDomainStage) -> &'static str {
    match stage {
        VcfDomainStage::Pca => "pca_report",
        VcfDomainStage::PopulationStructure => "population_structure_report",
        other => panic!("unsupported VCF eigensoft stage `{}`", other.as_str()),
    }
}

fn validate_required_inputs(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfEigensoftAdapterArtifact],
) -> Result<()> {
    for input in inputs {
        let resolved = repo_root.join(&input.path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "VCF eigensoft adapter for `{stage_id}` is missing required input `{}` at `{}`",
                input.artifact_id,
                input.path
            ));
        }
    }
    Ok(())
}

fn validate_command_steps(stage_id: &str, steps: &[VcfEigensoftAdapterCommandStep]) -> Result<()> {
    if steps.is_empty() {
        return Err(anyhow!(
            "VCF eigensoft adapter row `{stage_id}` must declare at least one command step"
        ));
    }
    let joined =
        steps.iter().flat_map(|step| step.argv.iter().cloned()).collect::<Vec<_>>().join(" ");
    for needle in ["convertf", "smartpca", ".geno", ".snp", ".ind", ".evec", ".eval"] {
        if !joined.contains(needle) {
            return Err(anyhow!(
                "VCF eigensoft adapter row `{stage_id}` is missing required token `{needle}`: {joined}"
            ));
        }
    }
    if joined.contains("--help") || joined.to_ascii_lowercase().contains("placeholder") {
        return Err(anyhow!(
            "VCF eigensoft adapter row `{stage_id}` still contains placeholder argv: {joined}"
        ));
    }
    Ok(())
}

fn run_missing_input_probe(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[VcfEigensoftAdapterArtifact],
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
            "VCF eigensoft adapter row `{stage_id}` unexpectedly accepted missing input `{}`",
            probe.artifact_id
        ),
        Err(error) => error.to_string(),
    };
    let passed = observed_error.contains(&expected_error_fragment);
    (probe.artifact_id, expected_error_fragment, observed_error, passed)
}

fn ensure_vcf_eigensoft_adapter_contract(
    registry_tool: &RegistryToolContract,
    rows: &[VcfEigensoftAdapterRow],
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

fn artifact(artifact_id: &str, role: &str, path: &str) -> VcfEigensoftAdapterArtifact {
    VcfEigensoftAdapterArtifact {
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
) -> VcfEigensoftAdapterCommandStep {
    VcfEigensoftAdapterCommandStep {
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

    use super::{render_vcf_eigensoft_adapter, DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_eigensoft_adapter_tracks_governed_rows() {
        let repo_root = repo_root();
        let report = render_vcf_eigensoft_adapter(
            &repo_root,
            PathBuf::from(DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH),
        )
        .expect("render VCF eigensoft adapter");

        assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_eigensoft_adapter.v1");
        assert_eq!(report.tool_id, "eigensoft");
        assert_eq!(report.tool_status, "experimental");
        assert_eq!(report.row_count, 2);
        assert_eq!(report.benchmark_ready_row_count, 1);
        assert_eq!(report.parser_output_row_count, 2);
        assert_eq!(report.normalized_metrics_row_count, 2);
        assert_eq!(report.conversion_output_row_count, 2);
        assert_eq!(report.pca_output_row_count, 2);

        let pca =
            report.rows.iter().find(|row| row.stage_id == "vcf.pca").expect("eigensoft pca row");
        assert_eq!(pca.benchmark_status, "benchmark_ready");
        assert_eq!(pca.normalized_metrics_artifact_id, "pca_report");
        assert!(
            pca.command_steps
                .iter()
                .flat_map(|step| step.argv.iter())
                .any(|part| part.contains("convertf")),
            "eigensoft pca row must keep convertf conversion"
        );

        let population_structure = report
            .rows
            .iter()
            .find(|row| row.stage_id == "vcf.population_structure")
            .expect("eigensoft population structure row");
        assert_eq!(
            population_structure.normalized_metrics_artifact_id,
            "population_structure_report"
        );
        assert!(
            population_structure
                .command_steps
                .iter()
                .flat_map(|step| step.argv.iter())
                .any(|part| part.contains("smartpca")),
            "eigensoft population structure row must keep smartpca execution"
        );
    }
}
