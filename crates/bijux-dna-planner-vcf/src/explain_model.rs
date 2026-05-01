use bijux_dna_domain_vcf::contracts::{
    VcfArtifactClass, VcfCallingModeContract, VcfCohortAnalysisBoundaryContract,
    VcfCohortValidationContract, VcfDamageFilterContract, VcfLikelihoodWorkflowContract,
    VcfNormalizationPolicyMatrixContract, VcfPanelBoundaryContract,
    VcfPhasingImputationBoundaryContract, VcfPopulationGuardrailContract,
    VcfProductionCorpusContract, VcfReportCoverageContract, VcfScientificDriftContract,
};
use bijux_dna_domain_vcf::taxonomy::CoverageRegime;
use serde::Serialize;

use crate::api::VcfPanelLock;
use crate::reference_context::ReferenceContextReport;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PlannerExplainStage {
    pub stage_id: String,
    pub selected_tool: String,
    pub reason: String,
    pub coverage_regime: CoverageRegime,
    pub params_surface: serde_json::Value,
    pub artifact_classes: Vec<VcfArtifactClass>,
    pub calling_mode_contract: Option<VcfCallingModeContract>,
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
    pub reference_context: ReferenceContextReport,
    pub selected_panel: Option<VcfPanelLock>,
    pub normalization_policy_matrix: VcfNormalizationPolicyMatrixContract,
    pub cohort_validation_contract: VcfCohortValidationContract,
    pub likelihood_workflow_contracts: Vec<VcfLikelihoodWorkflowContract>,
    pub panel_boundary_contracts: Vec<VcfPanelBoundaryContract>,
    pub phasing_imputation_boundary_contracts: Vec<VcfPhasingImputationBoundaryContract>,
    pub damage_filter_contract: VcfDamageFilterContract,
    pub population_guardrail_contracts: Vec<VcfPopulationGuardrailContract>,
    pub cohort_analysis_boundary_contracts: Vec<VcfCohortAnalysisBoundaryContract>,
    pub report_coverage_contract: VcfReportCoverageContract,
    pub production_corpus_contract: VcfProductionCorpusContract,
    pub scientific_drift_contract: VcfScientificDriftContract,
    pub decision_traces: Vec<serde_json::Value>,
    pub stages: Vec<PlannerExplainStage>,
}
