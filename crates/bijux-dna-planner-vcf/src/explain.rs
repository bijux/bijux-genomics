use bijux_dna_db_ref::{resolve_coverage_profile, resolve_reference_bundle};
use bijux_dna_domain_vcf::contracts::{
    stage_artifact_class_contract, vcf_calling_mode_contracts, vcf_panel_boundary_contracts,
    vcf_cohort_analysis_boundary_contracts, vcf_likelihood_workflow_contracts,
    vcf_phasing_imputation_boundary_contracts, vcf_population_guardrail_contracts,
    VCF_COHORT_VALIDATION_CONTRACT, VCF_DAMAGE_FILTER_CONTRACT,
    VCF_NORMALIZATION_POLICY_MATRIX_CONTRACT, VCF_PRODUCTION_CORPUS_CONTRACT,
    VCF_REPORT_COVERAGE_CONTRACT, VCF_SCIENTIFIC_DRIFT_CONTRACT,
};
use bijux_dna_stage_contract::StagePlanV1;

use crate::api::VcfPipelineInputs;
use crate::chunk_plan::plan_region_chunks;
use crate::coverage::{
    classify_coverage_regime, damage_aware_policy_for_regime, CoverageThresholds,
};
use crate::explain_model::{PlannerExplainStage, PlannerExplainV1};
use crate::reference_context::{resolve, resolve_panel_lock, ReferenceContextReport};

#[must_use]
pub fn explain_vcf_plan(inputs: &VcfPipelineInputs, plans: &[StagePlanV1]) -> PlannerExplainV1 {
    let resolved_context = resolve(inputs).ok();
    let bundle = resolve_reference_bundle(
        &inputs.species_context.species_id,
        &inputs.species_context.build_id,
    )
    .ok();
    let resolved_coverage_profile = resolve_coverage_profile(
        &inputs.species_context.species_id,
        &inputs.species_context.build_id,
    )
    .ok()
    .flatten();
    let (resolved_coverage_regime, coverage_resolution_reason, coverage_thresholds) =
        classify_coverage_regime(
            inputs.coverage_regime,
            inputs.mean_depth_x,
            resolved_coverage_profile.as_deref(),
        )
        .unwrap_or((
            inputs.coverage_regime,
            "coverage classifier unavailable; using caller-requested coverage_regime".to_string(),
            CoverageThresholds {
                gl_max_depth: 0.0,
                pseudohaploid_max_depth: 0.0,
                diploid_min_depth: 0.0,
            },
        ));
    let selected_panel = resolve_panel_lock(inputs).ok().flatten();
    let chunk_count =
        plan_region_chunks(&inputs.species_context, &inputs.chunking).map(|c| c.len()).unwrap_or(0);
    let stages = plans
        .iter()
        .map(|plan| {
            let stage = bijux_dna_domain_vcf::VcfDomainStage::try_from(plan.stage_id.to_string().as_str())
                .ok();
            let artifact_classes = stage
                .map(stage_artifact_class_contract)
                .map(|contract| contract.artifact_classes.iter().copied().collect::<Vec<_>>())
                .unwrap_or_default();
            let calling_mode_contract = stage.and_then(|stage_id| {
                vcf_calling_mode_contracts()
                    .iter()
                    .copied()
                    .find(|contract| contract.stage == stage_id)
            });
            PlannerExplainStage {
                stage_id: plan.stage_id.to_string(),
                selected_tool: plan.tool_id.to_string(),
                reason: plan.reason.summary.clone(),
                coverage_regime: resolved_coverage_regime,
                params_surface: plan.effective_params.clone(),
                artifact_classes,
                calling_mode_contract,
            }
        })
        .collect::<Vec<_>>();
    let reference_context = resolved_context
        .as_ref()
        .map(crate::reference_context::reference_context_report)
        .unwrap_or_else(|| ReferenceContextReport {
            schema_version: "bijux.vcf.reference_context_report.v1".to_string(),
            species_id: inputs.species_context.species_id.clone(),
            build_id: inputs.species_context.build_id.clone(),
            bundle_id: bundle
                .as_ref()
                .map(|value| value.bundle_id.clone())
                .unwrap_or_else(|| "unresolved".to_string()),
            bundle_lock_sha256: bundle
                .as_ref()
                .map(|value| value.bundle_lock_sha256.clone())
                .unwrap_or_else(|| "unresolved".to_string()),
            fasta_sha256: "unresolved".to_string(),
            contig_naming_scheme: "unresolved".to_string(),
            alias_count: 0,
            normalization_policy: bundle
                .as_ref()
                .map(|value| format!("{:?}", value.normalization_policy))
                .unwrap_or_else(|| "unresolved".to_string()),
            panel_id: "unresolved".to_string(),
            map_id: "unresolved".to_string(),
            vcf_index_required: true,
        });
    let planned_stage_ids = stages.iter().map(|stage| stage.stage_id.as_str()).collect::<Vec<_>>();
    let panel_boundary_contracts = vcf_panel_boundary_contracts()
        .iter()
        .copied()
        .filter(|contract| planned_stage_ids.iter().any(|stage_id| *stage_id == contract.stage.as_str()))
        .collect::<Vec<_>>();
    let phasing_imputation_boundary_contracts = vcf_phasing_imputation_boundary_contracts()
        .iter()
        .copied()
        .filter(|contract| planned_stage_ids.iter().any(|stage_id| *stage_id == contract.stage.as_str()))
        .collect::<Vec<_>>();
    let population_guardrail_contracts = vcf_population_guardrail_contracts()
        .iter()
        .copied()
        .filter(|contract| planned_stage_ids.iter().any(|stage_id| *stage_id == contract.stage.as_str()))
        .collect::<Vec<_>>();
    let cohort_analysis_boundary_contracts = vcf_cohort_analysis_boundary_contracts()
        .iter()
        .copied()
        .filter(|contract| planned_stage_ids.iter().any(|stage_id| *stage_id == contract.stage.as_str()))
        .collect::<Vec<_>>();
    let likelihood_workflow_contracts = vcf_likelihood_workflow_contracts()
        .iter()
        .copied()
        .filter(|contract| planned_stage_ids.iter().any(|stage_id| *stage_id == contract.stage.as_str()))
        .collect::<Vec<_>>();
    let panel_required = !panel_boundary_contracts.is_empty();
    PlannerExplainV1 {
        schema_version: "bijux.vcf.planner_explain.v1".to_string(),
        planner_version: crate::PLANNER_VERSION.to_string(),
        coverage_regime: inputs.coverage_regime,
        resolved_coverage_regime,
        coverage_resolution_reason,
        backend_selection_reason: format!(
            "selected backend family from resolved coverage regime {:?} and stage/tool compatibility",
            resolved_coverage_regime
        ),
        panel_selection_reason: if panel_required {
            "panel selected by build/license/ancestry policy".to_string()
        } else {
            "no panel required by resolved stage set".to_string()
        },
        map_selection_reason: format!(
            "map compatibility enforced by species/build/contig digest ({}/{})",
            inputs.species_context.species_id, inputs.species_context.build_id
        ),
        chunking_selection_reason: match resolved_coverage_regime {
            bijux_dna_domain_vcf::taxonomy::CoverageRegime::LowCovGl => {
                "lowcov_gl defaults to smaller windows for imputation stability".to_string()
            }
            bijux_dna_domain_vcf::taxonomy::CoverageRegime::Diploid => {
                "diploid defaults to larger chunks for throughput".to_string()
            }
            bijux_dna_domain_vcf::taxonomy::CoverageRegime::Pseudohaploid => {
                "pseudohaploid mode avoids diploid imputation chunking".to_string()
            }
        },
        resolved_reference_bundle_id: bundle
            .as_ref()
            .map(|b| b.bundle_id.clone())
            .unwrap_or_else(|| "unresolved".to_string()),
        resolved_reference_lock: bundle
            .as_ref()
            .map(|b| b.bundle_lock_sha256.clone())
            .unwrap_or_else(|| "unresolved".to_string()),
        resolved_coverage_profile,
        damage_aware_policy: damage_aware_policy_for_regime(resolved_coverage_regime),
        reference_context: reference_context.clone(),
        selected_panel,
        normalization_policy_matrix: VCF_NORMALIZATION_POLICY_MATRIX_CONTRACT,
        cohort_validation_contract: VCF_COHORT_VALIDATION_CONTRACT,
        likelihood_workflow_contracts,
        panel_boundary_contracts,
        phasing_imputation_boundary_contracts,
        damage_filter_contract: VCF_DAMAGE_FILTER_CONTRACT,
        population_guardrail_contracts,
        cohort_analysis_boundary_contracts,
        report_coverage_contract: VCF_REPORT_COVERAGE_CONTRACT,
        production_corpus_contract: VCF_PRODUCTION_CORPUS_CONTRACT,
        scientific_drift_contract: VCF_SCIENTIFIC_DRIFT_CONTRACT,
        decision_traces: vec![
            serde_json::json!({
                "id": "decision.backend_selection",
                "reason": "resolved_coverage_regime + stage/tool compatibility",
                "resolved_coverage_regime": resolved_coverage_regime,
                "why_stage_chosen": "stage order comes from resolved coverage regime defaults unless operator requested subset"
            }),
            serde_json::json!({
                "id": "decision.panel_selection",
                "reason": "build/license/ancestry constraints",
                "why_panel_chosen": "panel selected by policy from species/build and license constraints",
                "required_by_stage_set": panel_required,
            }),
            serde_json::json!({
                "id": "decision.map_selection",
                "reason": "species_id/build_id/contig_set_digest invariants",
            }),
            serde_json::json!({
                "id": "decision.chunking_selection",
                "reason": "coverage-regime-specific defaults",
                "chunk_count": chunk_count,
                "why_chunking_chosen": "chunk windows/overlap derive from coverage regime and chunking knobs",
            }),
            serde_json::json!({
                "id": "decision.imputation_accept",
                "reason": "qc thresholds and overlap stats gate acceptance",
            }),
            serde_json::json!({
                "id": "decision.reference_bundle_resolution",
                "reason": "resolve species/build -> canonical bundle + lock",
                "reference_context": reference_context.clone(),
            }),
            serde_json::json!({
                "id": "decision.coverage_regime",
                "reason": "configured thresholds + optional empirical mean_depth_x",
                "requested_coverage_regime": inputs.coverage_regime,
                "resolved_coverage_regime": resolved_coverage_regime,
                "mean_depth_x": inputs.mean_depth_x,
                "thresholds": {
                    "gl_max_depth": coverage_thresholds.gl_max_depth,
                    "pseudohaploid_max_depth": coverage_thresholds.pseudohaploid_max_depth,
                    "diploid_min_depth": coverage_thresholds.diploid_min_depth,
                }
            }),
            serde_json::json!({
                "id": "decision.damage_aware_genotype_logic",
                "reason": "regime-specific filtering/masking policy and UDG threshold profile",
                "policy": damage_aware_policy_for_regime(resolved_coverage_regime),
            }),
            serde_json::json!({
                "id": "decision.vcf_iteration16_contracts",
                "reason": "planner surfaces production contracts for normalization, cohort analysis, imputation boundaries, reporting, and scientific drift",
                "normalization_policy_ids": VCF_NORMALIZATION_POLICY_MATRIX_CONTRACT
                    .policy_rows
                    .iter()
                    .map(|row| row.policy_id)
                    .collect::<Vec<_>>(),
                "likelihood_stage_count": vcf_likelihood_workflow_contracts().len(),
                "production_corpus_case_count": VCF_PRODUCTION_CORPUS_CONTRACT.covered_cases.len(),
            }),
        ],
        stages,
    }
}
