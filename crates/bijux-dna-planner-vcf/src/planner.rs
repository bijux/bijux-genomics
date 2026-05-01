use std::collections::BTreeSet;

use anyhow::{anyhow, bail, Result};
use bijux_dna_core::contract::ExecutionGraph;
use bijux_dna_core::prelude::ArtifactRole;
use bijux_dna_domain_vcf::contracts::{
    refuse_unsupported_regime_transition, vcf_panel_boundary_contracts,
    VCF_COHORT_VALIDATION_CONTRACT,
};
use bijux_dna_domain_vcf::taxonomy::{
    validate_downstream_transition, CoverageRegime, VcfDomainStage,
};
use bijux_dna_stage_contract::StagePlanV1;

use crate::api::VcfPipelineInputs;
use crate::chunk_plan::plan_region_chunks;
use crate::reference_context::ResolvedPlanningContext;
use crate::stage_sequence::resolve_requested_stages;
use crate::tool_selection::{choose_tool, validate_selected_tool};
use crate::workspace_config::{load_registry_tools, load_required_tools};

/// # Errors
/// Returns an error when stage selection is invalid for downstream execution.
pub fn plan_vcf_stage_plans(inputs: &VcfPipelineInputs) -> Result<Vec<StagePlanV1>> {
    let required_tools = load_required_tools()?;
    let registry_tools = load_registry_tools()?;
    crate::input_policy::validate(inputs)?;
    let ResolvedPlanningContext {
        species: resolved_species,
        bundle,
        reference_bank: _reference_bank,
        contig_map: _contig_map,
        panel_catalog,
        map_catalog,
        resolved_coverage,
        selected_panel,
    } = crate::reference_context::resolve(inputs)?;
    let chunks = plan_region_chunks(&inputs.species_context, &inputs.chunking)?;
    let stages = resolve_requested_stages(&inputs.requested_stages, resolved_coverage)?;
    validate_stage_tool_override_keys(inputs, &stages)?;
    validate_stage_param_override_keys(inputs, &stages)?;
    validate_stage_coverage_support(&stages, resolved_coverage)?;
    validate_stage_ordering(&stages)?;
    validate_panel_and_cohort_contracts(
        inputs,
        &stages,
        selected_panel.is_some(),
        &map_catalog.id,
    )?;

    if stages.contains(&VcfDomainStage::Demography) && !stages.contains(&VcfDomainStage::Ibd) {
        bail!("vcf.demography requires vcf.ibd in requested/default stage set");
    }
    let requires_diploid_imputation = stages.iter().any(|s| {
        matches!(s, VcfDomainStage::Phasing | VcfDomainStage::Imputation | VcfDomainStage::Impute)
    });
    if requires_diploid_imputation && !resolved_species.supported_features.imputation {
        bail!(
            "planner refusal: species/build {}:{} does not support imputation",
            inputs.species_context.species_id,
            inputs.species_context.build_id
        );
    }
    if requires_diploid_imputation && inputs.species_context.par_policy == "unsupported" {
        bail!(
            "planner refusal: sex/PAR policy unsupported for imputation on {}:{}",
            inputs.species_context.species_id,
            inputs.species_context.build_id
        );
    }
    refuse_unsupported_regime_transition(resolved_coverage, requires_diploid_imputation)?;

    let stage_list = stages.clone();
    let mut seen = BTreeSet::new();
    let mut plans = Vec::new();
    let mut current_vcf = inputs.vcf.clone();
    for stage in stage_list {
        if !seen.insert(stage.as_str().to_string()) {
            continue;
        }
        let (tool, selection_rule) =
            choose_tool(stage, inputs, resolved_coverage, &panel_catalog, &stages)?;
        if !required_tools.contains(&tool) {
            bail!(
                "planner refusal: tool {} for {} is not declared in required_tools_vcf(_downstream).toml",
                tool,
                stage.as_str()
            );
        }
        if !registry_tools.contains(&tool) {
            bail!(
                "planner refusal: tool {} for {} is missing from tool_registry_vcf(_downstream).toml",
                tool,
                stage.as_str()
            );
        }
        validate_selected_tool(stage, &tool, resolved_coverage, &panel_catalog, &map_catalog)?;
        let plan = crate::stage_plan::build_stage_plan(
            stage,
            &current_vcf,
            &inputs.out_dir,
            &tool,
            resolved_coverage,
            selected_panel.as_ref(),
            &map_catalog,
            &bundle,
            &inputs.species_context.species_id,
            &inputs.species_context.build_id,
            &inputs.stage_param_overrides,
            &inputs.chunking,
            &chunks,
            &selection_rule,
        )?;
        if let Some(out) = plan
            .io
            .outputs
            .iter()
            .find(|output| output.role == ArtifactRole::Reads)
            .filter(|_| stage != VcfDomainStage::PrepareReferencePanel)
        {
            current_vcf = out.path.clone();
        }
        plans.push(plan);
    }
    if plans.is_empty() {
        return Err(anyhow!("no VCF stage plans generated"));
    }
    Ok(plans)
}

fn validate_stage_tool_override_keys(
    inputs: &VcfPipelineInputs,
    stages: &[VcfDomainStage],
) -> Result<()> {
    for stage_id in inputs.stage_tool_overrides.keys() {
        let stage = VcfDomainStage::try_from(stage_id.as_str())
            .map_err(|err| anyhow!("unknown stage_tool_overrides key {stage_id}: {err}"))?;
        if !stages.contains(&stage) {
            bail!("stage_tool_overrides key {} is not in the resolved stage set", stage.as_str());
        }
    }
    Ok(())
}

fn validate_stage_param_override_keys(
    inputs: &VcfPipelineInputs,
    stages: &[VcfDomainStage],
) -> Result<()> {
    for stage_id in inputs.stage_param_overrides.keys() {
        let stage = VcfDomainStage::try_from(stage_id.as_str())
            .map_err(|err| anyhow!("unknown stage_param_overrides key {stage_id}: {err}"))?;
        if !stages.contains(&stage) {
            bail!("stage_param_overrides key {} is not in the resolved stage set", stage.as_str());
        }
    }
    Ok(())
}

fn validate_stage_coverage_support(
    stages: &[VcfDomainStage],
    coverage: CoverageRegime,
) -> Result<()> {
    for stage in stages {
        if !stage.taxonomy().coverage_regimes.contains(&coverage) {
            bail!(
                "{} is not supported for resolved coverage regime {:?}",
                stage.as_str(),
                coverage
            );
        }
    }
    Ok(())
}

fn validate_stage_ordering(stages: &[VcfDomainStage]) -> Result<()> {
    for pair in stages.windows(2) {
        validate_downstream_transition(pair[0], pair[1])?;
    }
    Ok(())
}

fn validate_panel_and_cohort_contracts(
    inputs: &VcfPipelineInputs,
    stages: &[VcfDomainStage],
    panel_selected: bool,
    resolved_map_id: &str,
) -> Result<()> {
    let panel_boundary_stages = vcf_panel_boundary_contracts()
        .iter()
        .filter(|contract| stages.contains(&contract.stage))
        .collect::<Vec<_>>();
    if !panel_boundary_stages.is_empty() && !panel_selected && inputs.panel_id.is_none() {
        let stage_list = panel_boundary_stages
            .iter()
            .map(|contract| contract.stage.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        bail!(
            "planner refusal: explicit panel identity is required for {stage_list}; provide panel_lock or panel_id"
        );
    }
    if !panel_boundary_stages.is_empty() && resolved_map_id.trim().is_empty() {
        bail!("planner refusal: phasing/imputation workflows require a resolved genetic map");
    }

    let requires_cohort_validation = stages
        .iter()
        .any(|stage| VCF_COHORT_VALIDATION_CONTRACT.cohort_analysis_stages.contains(stage));
    if requires_cohort_validation {
        if !inputs.entry_vcf_invariants.sample_ids_non_empty_unique {
            bail!("planner refusal: cohort analysis requires unique non-empty sample IDs");
        }
        if !inputs.entry_vcf_invariants.ploidy_constraints_ok {
            bail!("planner refusal: cohort analysis requires declared sex/ploidy assumptions");
        }
        if !inputs.panel_map_invariants.sample_count_ok {
            bail!("planner refusal: cohort analysis requires cohort sample-count readiness");
        }
    }
    Ok(())
}

/// # Errors
/// Returns an error when graph materialization fails.
pub fn plan_vcf_pipeline(inputs: &VcfPipelineInputs) -> Result<ExecutionGraph> {
    let plans = plan_vcf_stage_plans(inputs)?;
    let resolved_coverage = crate::reference_context::resolve(inputs)?.resolved_coverage;
    crate::execution_graph::build_vcf_pipeline_graph(inputs, resolved_coverage, &plans)
}

/// Backward-compatible entrypoint retained for older callers.
///
/// # Errors
/// Returns an error when minimal plan generation fails.
pub fn plan_vcf_minimal(inputs: &VcfPipelineInputs) -> Result<ExecutionGraph> {
    let mut compat = inputs.clone();
    compat.coverage_regime = CoverageRegime::Diploid;
    compat.requested_stages =
        Some(vec!["vcf.call".to_string(), "vcf.filter".to_string(), "vcf.stats".to_string()]);
    plan_vcf_pipeline(&compat)
}
