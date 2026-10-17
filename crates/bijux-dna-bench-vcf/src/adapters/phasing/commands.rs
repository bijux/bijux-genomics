//! Owner: bijux-dna-bench-vcf
//! phasing adapter commands contracts.
use super::{
    anyhow, load_governed_vcf_fixture_inputs, materialize_indexed_vcf_input,
    materialize_panel_inputs, run_missing_input_probe, validate_command_steps,
    validate_required_inputs, vcf_domain_stage_expected_output_ids, MaterializedPanelInputs, Path,
    PathBuf, RegistryToolContract, Result, StageContract, VcfDomainStage,
    VcfPhasingFamilyAdapterArtifact, VcfPhasingFamilyAdapterCommandStep,
    VcfPhasingFamilyAdapterRow, VcfStageCatalogRow, VcfStageMatrixRow,
    VCF_BEAGLE_ADAPTER_SCHEMA_VERSION, VCF_EAGLE_ADAPTER_SCHEMA_VERSION,
    VCF_SHAPEIT5_ADAPTER_SCHEMA_VERSION, VCF_SHAPEIT_ADAPTER_SCHEMA_VERSION,
};

pub(super) fn build_phasing_family_row(
    repo_root: &Path,
    registry_tool: &RegistryToolContract,
    stage_id: &str,
    catalog_row: &VcfStageCatalogRow,
    matrix_row: Option<&VcfStageMatrixRow>,
) -> Result<VcfPhasingFamilyAdapterRow> {
    let stage = VcfDomainStage::try_from(stage_id)
        .map_err(|error| anyhow!("unknown VCF stage `{stage_id}`: {error}"))?;
    let identity = crate::inputs::resolve_adapter_binding_identity(
        stage,
        stage_id,
        &registry_tool.tool_id,
        matrix_row,
        "vcf_cohort_with_panel",
    )?;
    let output_root = format!(
        "artifacts/benchmark-smoke/readiness/adapters/{}/{}",
        registry_tool.tool_id, stage_id
    );
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
        &indexed_input_vcf_path,
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
        adapter_id: identity.adapter,
        parser_id: identity.parser,
        corpus_id: identity.corpus,
        asset_profile_id: identity.asset_profile,
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

pub(super) fn build_stage_contract(
    tool_id: &str,
    output_root: &str,
    output_prefix: &str,
    input_vcf_path: &str,
    materialized_inputs: &MaterializedPanelInputs,
) -> Result<StageContract> {
    let phased_vcf_path = format!("{output_root}/phased.vcf.gz");
    let phased_vcf_tbi_path = format!("{phased_vcf_path}.tbi");
    let phase_block_stats_path = format!("{output_root}/phase_block_stats.tsv");
    let switch_error_proxy_path = format!("{output_root}/switch_error_proxy.tsv");
    let phasing_qc_path = format!("{output_root}/phasing_qc.json");
    let phasing_manifest_path = format!("{output_root}/phasing_manifest.json");
    let logs_path = format!("{output_root}/logs.txt");
    let (command_contract_source, phase_step_argv) = super::invocation::build(
        tool_id,
        output_prefix,
        input_vcf_path,
        materialized_inputs,
        &phased_vcf_path,
        &logs_path,
    )?;

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

pub(super) fn schema_version_for_tool(tool_id: &str) -> &'static str {
    match tool_id {
        "shapeit5" => VCF_SHAPEIT5_ADAPTER_SCHEMA_VERSION,
        "shapeit" => VCF_SHAPEIT_ADAPTER_SCHEMA_VERSION,
        "eagle" => VCF_EAGLE_ADAPTER_SCHEMA_VERSION,
        "beagle" => VCF_BEAGLE_ADAPTER_SCHEMA_VERSION,
        other => panic!("unsupported VCF phasing-family schema tool `{other}`"),
    }
}

pub(super) fn artifact(
    artifact_id: &str,
    role: &str,
    path: &str,
) -> VcfPhasingFamilyAdapterArtifact {
    VcfPhasingFamilyAdapterArtifact {
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
