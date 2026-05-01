use anyhow::{anyhow, bail, Result};
use bijux_dna_db_ref::ReferenceBundle;
use bijux_dna_domain_vcf::contracts::{
    vcf_cohort_analysis_boundary_contracts, vcf_likelihood_workflow_contracts,
    vcf_phasing_imputation_boundary_contracts, vcf_population_guardrail_contracts,
    VcfCohortAnalysisBoundaryContract, VcfLikelihoodWorkflowContract, VcfNormalizationPolicyRow,
    VcfPhasingImputationBoundaryContract, VcfPopulationGuardrailContract,
    VCF_DAMAGE_FILTER_CONTRACT, VCF_NORMALIZATION_POLICY_MATRIX_CONTRACT,
    VCF_REPORT_COVERAGE_CONTRACT,
};
use bijux_dna_domain_vcf::taxonomy::{CoverageRegime, VcfDomainStage};
use std::collections::BTreeSet;

use crate::api::{ChunkPlanSettings, VcfPanelLock};
use crate::chunk_plan::RegionChunkPlan;
use crate::workspace_config;

fn normalization_policy_for_coverage(
    coverage: CoverageRegime,
) -> &'static VcfNormalizationPolicyRow {
    let policy_id = match coverage {
        CoverageRegime::Diploid => "diploid_production",
        CoverageRegime::LowCovGl => "lowcov_gl_production",
        CoverageRegime::Pseudohaploid => "pseudohaploid_production",
    };
    VCF_NORMALIZATION_POLICY_MATRIX_CONTRACT
        .policy_rows
        .iter()
        .find(|row| row.policy_id == policy_id)
        .expect("normalization policy must exist for every coverage regime")
}

fn likelihood_contract(
    stage: VcfDomainStage,
    coverage: CoverageRegime,
) -> Option<&'static VcfLikelihoodWorkflowContract> {
    vcf_likelihood_workflow_contracts()
        .iter()
        .find(|contract| contract.stage == stage && contract.coverage_regime == coverage)
}

fn phase_impute_boundary(
    stage: VcfDomainStage,
) -> Option<&'static VcfPhasingImputationBoundaryContract> {
    vcf_phasing_imputation_boundary_contracts().iter().find(|contract| contract.stage == stage)
}

fn population_guardrail(stage: VcfDomainStage) -> Option<&'static VcfPopulationGuardrailContract> {
    vcf_population_guardrail_contracts().iter().find(|contract| contract.stage == stage)
}

fn cohort_boundary(stage: VcfDomainStage) -> Option<&'static VcfCohortAnalysisBoundaryContract> {
    vcf_cohort_analysis_boundary_contracts().iter().find(|contract| contract.stage == stage)
}

pub(crate) fn stage_params(
    stage: VcfDomainStage,
    tool: &str,
    coverage: CoverageRegime,
    panel: Option<&VcfPanelLock>,
    map: &bijux_dna_db_ref::MapCatalogEntry,
    chunking: &ChunkPlanSettings,
    chunks: &[RegionChunkPlan],
) -> serde_json::Value {
    let normalization_policy = normalization_policy_for_coverage(coverage);
    match stage {
        VcfDomainStage::PrepareReferencePanel => serde_json::json!({
            "schema_version": "bijux.vcf.prepare_reference_panel.params.v1",
            "tool": tool,
            "panel_lock": panel,
            "panel_id": panel.map(|value| value.panel_id.as_str()),
            "panel_identity_required": true,
            "panel_identity_source": panel.map(|_| "panel_lock").unwrap_or("catalog_default"),
            "map_id": map.id,
            "map_coordinate_system": map.compatibility.coordinate_system,
            "normalize": normalization_policy.left_normalize,
            "normalization_policy": normalization_policy,
            "license_policy": "panel_license_must_match_operator_constraints",
            "require_bgzip_tabix": true,
            "prepared_outputs": ["panel_manifest", "overlap_report", "chunks_plan"],
        }),
        VcfDomainStage::Phasing => match tool {
            "shapeit5" => {
                let boundary = phase_impute_boundary(stage).expect("phasing boundary contract");
                serde_json::json!({
                    "schema_version":"bijux.vcf.phasing.params.v1",
                    "tool":"shapeit5",
                    "panel_id": panel.map(|value| value.panel_id.as_str()),
                    "panel_identity_required": true,
                    "panel_build_required": panel.map(|value| value.reference_build.as_str()),
                    "window_cM":2.0,
                    "pbwt_depth":8,
                    "threads":8,
                    "seed":42,
                    "map_id":map.id,
                    "map_coordinate_system":map.compatibility.coordinate_system,
                    "allow_gl_only_input":false,
                    "chunking":chunking,
                    "chunks_plan":chunks,
                    "confidence_outputs": boundary.required_outputs,
                    "boundary_contract": boundary,
                })
            }
            "eagle" => {
                let boundary = phase_impute_boundary(stage).expect("phasing boundary contract");
                serde_json::json!({
                    "schema_version":"bijux.vcf.phasing.params.v1",
                    "tool":"eagle",
                    "panel_id": panel.map(|value| value.panel_id.as_str()),
                    "panel_identity_required": true,
                    "panel_build_required": panel.map(|value| value.reference_build.as_str()),
                    "max_iterations":10,
                    "use_reference":true,
                    "threads":8,
                    "seed":42,
                    "map_id":map.id,
                    "map_coordinate_system":map.compatibility.coordinate_system,
                    "allow_gl_only_input":false,
                    "chunking":chunking,
                    "chunks_plan":chunks,
                    "confidence_outputs": boundary.required_outputs,
                    "boundary_contract": boundary,
                })
            }
            _ => {
                let boundary = phase_impute_boundary(stage).expect("phasing boundary contract");
                let likelihood =
                    likelihood_contract(VcfDomainStage::CallGl, CoverageRegime::LowCovGl);
                serde_json::json!({
                    "schema_version":"bijux.vcf.phasing.params.v1",
                    "tool":"beagle",
                    "panel_id": panel.map(|value| value.panel_id.as_str()),
                    "panel_identity_required": true,
                    "panel_build_required": panel.map(|value| value.reference_build.as_str()),
                    "burnin":6,
                    "iterations":12,
                    "threads":8,
                    "seed":42,
                    "map_id":map.id,
                    "map_coordinate_system":map.compatibility.coordinate_system,
                    "allow_gl_only_input":coverage==CoverageRegime::LowCovGl,
                    "chunking":chunking,
                    "chunks_plan":chunks,
                    "confidence_outputs": boundary.required_outputs,
                    "boundary_contract": boundary,
                    "gl_input_contract": likelihood,
                })
            }
        },
        VcfDomainStage::Imputation | VcfDomainStage::Impute => match tool {
            "glimpse" => {
                let boundary = phase_impute_boundary(stage).expect("imputation boundary contract");
                serde_json::json!({
                    "schema_version":"bijux.vcf.impute.params.v1",
                    "tool":"glimpse",
                    "panel_id": panel.map(|value| value.panel_id.as_str()),
                    "panel_identity_required": true,
                    "panel_build_required": panel.map(|value| value.reference_build.as_str()),
                    "map_id": map.id,
                    "map_coordinate_system": map.compatibility.coordinate_system,
                    "window_size_mb":2.0,
                    "buffer_mb":0.2,
                    "emit_gp":true,
                    "chunking":chunking,
                    "chunks_plan":chunks,
                    "confidence_outputs": boundary.required_outputs,
                    "boundary_contract": boundary,
                    "imputation_acceptance_mode": "qc_gated",
                })
            }
            "impute5" => {
                let boundary = phase_impute_boundary(stage).expect("imputation boundary contract");
                serde_json::json!({
                    "schema_version":"bijux.vcf.impute.params.v1",
                    "tool":"impute5",
                    "panel_id": panel.map(|value| value.panel_id.as_str()),
                    "panel_identity_required": true,
                    "panel_build_required": panel.map(|value| value.reference_build.as_str()),
                    "map_id": map.id,
                    "map_coordinate_system": map.compatibility.coordinate_system,
                    "ne":20000,
                    "r2_threshold":0.3,
                    "chunking":chunking,
                    "chunks_plan":chunks,
                    "confidence_outputs": boundary.required_outputs,
                    "boundary_contract": boundary,
                    "imputation_acceptance_mode": "qc_gated",
                })
            }
            "minimac4" => {
                let boundary = phase_impute_boundary(stage).expect("imputation boundary contract");
                serde_json::json!({
                    "schema_version":"bijux.vcf.impute.params.v1",
                    "tool":"minimac4",
                    "panel_id": panel.map(|value| value.panel_id.as_str()),
                    "panel_identity_required": true,
                    "panel_build_required": panel.map(|value| value.reference_build.as_str()),
                    "map_id": map.id,
                    "map_coordinate_system": map.compatibility.coordinate_system,
                    "rounds":5,
                    "states":200,
                    "min_rsq":0.3,
                    "chunking":chunking,
                    "chunks_plan":chunks,
                    "confidence_outputs": boundary.required_outputs,
                    "boundary_contract": boundary,
                    "imputation_acceptance_mode": "qc_gated",
                })
            }
            _ => {
                let boundary = phase_impute_boundary(stage).expect("imputation boundary contract");
                serde_json::json!({
                    "schema_version":"bijux.vcf.impute.params.v1",
                    "tool":"beagle",
                    "panel_id": panel.map(|value| value.panel_id.as_str()),
                    "panel_identity_required": true,
                    "panel_build_required": panel.map(|value| value.reference_build.as_str()),
                    "map_id": map.id,
                    "map_coordinate_system": map.compatibility.coordinate_system,
                    "ne":10000,
                    "impute":true,
                    "chunking":chunking,
                    "chunks_plan":chunks,
                    "confidence_outputs": boundary.required_outputs,
                    "boundary_contract": boundary,
                    "imputation_acceptance_mode": "qc_gated",
                })
            }
        },
        VcfDomainStage::CallGl | VcfDomainStage::CallPseudohaploid => {
            let likelihood = likelihood_contract(stage, coverage);
            serde_json::json!({
                "schema_version": "bijux.vcf.likelihood_workflow.params.v1",
                "tool": tool,
                "coverage_regime": coverage,
                "likelihood_contract": likelihood,
                "normalization_policy_id": normalization_policy.policy_id,
            })
        }
        VcfDomainStage::GlPropagation => serde_json::json!({
            "schema_version": "bijux.vcf.gl_propagation.params.v1",
            "tool": tool,
            "retain_fields": ["GL", "PL", "GP"],
            "propagation_scope": "post_filter_pre_impute",
            "likelihood_contract": likelihood_contract(stage, coverage),
            "output_caveats": likelihood_contract(stage, coverage).map(|contract| contract.output_caveats),
            "emit_bcf": true,
        }),
        VcfDomainStage::DamageFilter => serde_json::json!({
            "schema_version": "bijux.vcf.damage_filter.params.v1",
            "tool": tool,
            "mask_ct_ga_transitions": true,
            "pmd_threshold": 3,
            "damage_audit": true,
            "action_scope": "site_and_genotype",
            "execution_mode": "enforced",
            "damage_contract": VCF_DAMAGE_FILTER_CONTRACT,
        }),
        VcfDomainStage::PopulationStructure | VcfDomainStage::Pca | VcfDomainStage::Admixture => {
            let guardrail = population_guardrail(stage).expect("population guardrail contract");
            serde_json::json!({
                "schema_version": "bijux.vcf.population_structure.params.v1",
                "tool": tool,
                "ld_prune": true,
                "ld_pruning_policy": "plink2_indep_pairwise",
                "maf_threshold": 0.01,
                "missingness_threshold": 0.1,
                "components": 10,
                "sample_metadata_manifest_required": true,
                "sample_inclusion_policy": "header_samples_intersect_metadata_manifest",
                "emit_artifacts": ["eigenvec", "eigenval", "cluster_assignments"],
                "interpretation_caveats": guardrail.report_caveats,
                "guardrail_contract": guardrail,
            })
        }
        VcfDomainStage::Ibd => serde_json::json!({
            "schema_version": "bijux.vcf.ibd.params.v1",
            "tool": tool,
            "min_samples": 2,
            "sample_constraints": {"allow_related": true, "max_missing": 0.05},
            "min_segment_cm": 3.0,
            "cohort_boundary_contract": cohort_boundary(stage),
        }),
        VcfDomainStage::Roh => serde_json::json!({
            "schema_version": "bijux.vcf.roh.params.v1",
            "tool": tool,
            "min_kb": 500,
            "max_gap_kb": 100,
            "het_per_window": 1,
            "cohort_boundary_contract": cohort_boundary(stage),
        }),
        VcfDomainStage::Demography => serde_json::json!({
            "schema_version": "bijux.vcf.demography.params.v1",
            "tool": tool,
            "requires_ibd_input": true,
            "generation_time_years": 29,
            "cohort_boundary_contract": cohort_boundary(stage),
        }),
        VcfDomainStage::Postprocess => serde_json::json!({
            "schema_version": "bijux.vcf.postprocess.params.v1",
            "tool": tool,
            "coverage_regime": coverage,
            "normalization_policy": normalization_policy,
            "retain_raw_view": true,
            "report_sections": VCF_REPORT_COVERAGE_CONTRACT.report_sections,
        }),
        VcfDomainStage::Qc | VcfDomainStage::Stats => serde_json::json!({
            "schema_version": "bijux.vcf.reporting.params.v1",
            "tool": tool,
            "coverage_regime": coverage,
            "report_sections": VCF_REPORT_COVERAGE_CONTRACT.report_sections,
            "per_sample_sections": VCF_REPORT_COVERAGE_CONTRACT.per_sample_sections,
        }),
        _ => serde_json::json!({
            "schema_version": "bijux.vcf.stage.params.v1",
            "tool": tool,
            "coverage_regime": coverage,
        }),
    }
}

fn allowed_params_for_stage(stage_id: &str) -> Result<BTreeSet<String>> {
    workspace_config::allowed_params_for_stage(stage_id)
}

pub(crate) fn validate_generated_stage_params(
    stage_id: &str,
    params: &serde_json::Value,
) -> Result<()> {
    let allowed = allowed_params_for_stage(stage_id)?;
    if allowed.is_empty() {
        bail!("no param registry entry for {stage_id} in param_registry_downstream.toml");
    }
    let obj = params
        .as_object()
        .ok_or_else(|| anyhow!("internal planner params for {stage_id} must be a JSON object"))?;
    for key in obj.keys() {
        if !allowed.contains(key) {
            bail!(
                "unregistered knob for {stage_id}: `{key}` (add to param_registry_downstream.toml)"
            );
        }
    }
    Ok(())
}

pub(crate) fn apply_stage_param_overrides(
    stage_id: &str,
    mut base: serde_json::Value,
    overrides: Option<&serde_json::Value>,
) -> Result<serde_json::Value> {
    let Some(override_value) = overrides else {
        return Ok(base);
    };
    let obj = override_value
        .as_object()
        .ok_or_else(|| anyhow!("stage_param_overrides[{stage_id}] must be a JSON object"))?;
    let allowed = allowed_params_for_stage(stage_id)?;
    for key in obj.keys() {
        if !allowed.contains(key) {
            bail!("unknown knob for {stage_id}: `{key}` (not in param_registry_downstream.toml)");
        }
    }
    let base_obj = base
        .as_object_mut()
        .ok_or_else(|| anyhow!("internal planner params must be JSON object"))?;
    for (k, v) in obj {
        base_obj.insert(k.clone(), v.clone());
    }
    Ok(base)
}

pub(crate) fn attach_reference_provenance(
    params: serde_json::Value,
    species_id: &str,
    build_id: &str,
    bundle: &ReferenceBundle,
) -> serde_json::Value {
    let mut obj = match params {
        serde_json::Value::Object(map) => map,
        other => {
            let mut map = serde_json::Map::new();
            map.insert("value".to_string(), other);
            map
        }
    };
    obj.insert(
        "reference_provenance".to_string(),
        serde_json::json!(bijux_dna_db_ref::reference_provenance(species_id, build_id, bundle)),
    );
    serde_json::Value::Object(obj)
}
