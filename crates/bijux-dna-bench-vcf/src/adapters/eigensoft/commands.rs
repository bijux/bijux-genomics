//! Owner: bijux-dna-bench-vcf
//! eigensoft adapter commands contracts.
use super::{
    anyhow, load_governed_vcf_fixture_inputs, run_missing_input_probe, validate_command_steps,
    validate_required_inputs, vcf_domain_stage_expected_output_ids, Path, PathBuf,
    RegistryToolContract, Result, StageContract, VcfDomainStage, VcfEigensoftAdapterArtifact,
    VcfEigensoftAdapterCommandStep, VcfEigensoftAdapterRow, VcfStageCatalogRow, VcfStageMatrixRow,
    GOVERNED_COHORT_VCF_PATH,
};

pub(super) fn build_eigensoft_row(
    repo_root: &Path,
    registry_tool: &RegistryToolContract,
    stage_id: &str,
    catalog_row: &VcfStageCatalogRow,
    matrix_row: Option<&VcfStageMatrixRow>,
) -> Result<VcfEigensoftAdapterRow> {
    let stage = VcfDomainStage::try_from(stage_id)
        .map_err(|error| anyhow!("unknown VCF stage `{stage_id}`: {error}"))?;
    let identity = crate::inputs::resolve_adapter_binding_identity(
        stage,
        stage_id,
        &registry_tool.tool_id,
        matrix_row,
        "vcf_cohort",
    )?;
    let output_root = format!(
        "artifacts/benchmark-smoke/readiness/adapters/{}/{}",
        registry_tool.tool_id, stage_id
    );
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
        adapter_id: identity.adapter,
        parser_id: identity.parser,
        corpus_id: identity.corpus,
        asset_profile_id: identity.asset_profile,
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

pub(super) fn build_stage_contract(
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
    let command_steps = super::invocation::CommandFiles {
        convertf_par: &convertf_par_path,
        eigenval: &eigenval_path,
        eigenvec: &eigenvec_path,
        geno: &geno_path,
        ind: &ind_path,
        smartpca_log: &smartpca_log_path,
        smartpca_par: &smartpca_par_path,
        snp: &snp_path,
    }
    .steps();

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

pub(super) fn stage_output_name_hint(stage: VcfDomainStage) -> &'static str {
    match stage {
        VcfDomainStage::Pca => "pca_report",
        VcfDomainStage::PopulationStructure => "population_structure_report",
        other => panic!("unsupported VCF eigensoft stage `{}`", other.as_str()),
    }
}

pub(super) fn artifact(artifact_id: &str, role: &str, path: &str) -> VcfEigensoftAdapterArtifact {
    VcfEigensoftAdapterArtifact {
        artifact_id: artifact_id.to_string(),
        role: role.to_string(),
        path: path.to_string(),
    }
}

pub(super) fn step(
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

pub(super) fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

pub(super) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
