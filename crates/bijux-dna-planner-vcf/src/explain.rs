use bijux_dna_db_ref::{resolve_coverage_profile, resolve_reference_bundle};
use bijux_dna_stage_contract::StagePlanV1;

use crate::coverage::{classify_coverage_regime, damage_aware_policy_for_regime, CoverageThresholds};
use crate::models::{PlannerExplainStage, PlannerExplainV1, VcfPipelineInputs};
use crate::planner::resolve_panel_lock;

#[must_use]
pub fn explain_vcf_plan(inputs: &VcfPipelineInputs, plans: &[StagePlanV1]) -> PlannerExplainV1 {
    let bundle = resolve_reference_bundle(&inputs.species_context.species_id, &inputs.species_context.build_id).ok();
    let resolved_coverage_profile = resolve_coverage_profile(&inputs.species_context.species_id, &inputs.species_context.build_id)
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
    let chunk_count = super::planner::plan_region_chunks(&inputs.species_context, &inputs.chunking)
        .map(|c| c.len())
        .unwrap_or(0);
    let stages = plans
        .iter()
        .map(|plan| PlannerExplainStage {
            stage_id: plan.stage_id.to_string(),
            selected_tool: plan.tool_id.to_string(),
            reason: plan.reason.summary.clone(),
            coverage_regime: resolved_coverage_regime,
            params_surface: plan.effective_params.clone(),
        })
        .collect::<Vec<_>>();
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
        panel_selection_reason: if selected_panel.is_some() {
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
        selected_panel,
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
        ],
        stages,
    }
}
