use bijux_dna_domain_vcf::taxonomy::CoverageRegime;
use serde::Serialize;

use crate::api::VcfPanelLock;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PlannerExplainStage {
    pub stage_id: String,
    pub selected_tool: String,
    pub reason: String,
    pub coverage_regime: CoverageRegime,
    pub params_surface: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PlannerExplainV1 {
    pub schema_version: String,
    pub planner_version: String,
    pub coverage_regime: CoverageRegime,
    pub backend_selection_reason: String,
    pub panel_selection_reason: String,
    pub map_selection_reason: String,
    pub chunking_selection_reason: String,
    pub resolved_reference_bundle_id: String,
    pub resolved_reference_lock: String,
    pub resolved_coverage_profile: Option<String>,
    pub resolved_coverage_regime: CoverageRegime,
    pub coverage_resolution_reason: String,
    pub damage_aware_policy: serde_json::Value,
    pub selected_panel: Option<VcfPanelLock>,
    pub decision_traces: Vec<serde_json::Value>,
    pub stages: Vec<PlannerExplainStage>,
}
