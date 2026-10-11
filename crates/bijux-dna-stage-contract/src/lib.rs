#![allow(
    clippy::default_trait_access,
    clippy::expect_used,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::unwrap_used
)]

pub mod execution_plan;
pub mod executor_registry;
pub mod plan_run;
pub mod stage_plan;
pub mod stage_plugin;
pub mod tool_binding;

pub use execution_plan::{
    default_edges_for_stages, lint_execution_plan, ExecutionPlan, PlanEdge, PlanValidationContext,
};
pub use executor_registry::{
    entries, entry, has_executor, ReadinessBadge, StageDomain, StageExecutorEntry,
};
pub use plan_run::{
    artifact_kind_schema, build_run_execution_plan, build_stage_plan, build_tool_execution_spec,
    evaluate_stage_admission, validate_stage_contract, validate_stage_outputs, DryRunExecutor,
    Executor, PlannerContractV1, RunExecutionPlan, StageAdmissionOutcomeV1,
    StageAdmissionRequestV1, StageRefusalV1,
};
pub use stage_plan::{
    execution_step_from_stage_plan, execution_step_from_stage_plan_with_step_id,
    PlanDecisionReason, PlanReasonKind, PlannedArtifactV1, StageArtifactPromiseV1, StagePlanJsonV1,
    StagePlanV1, StageProvenanceV1,
};
pub use stage_plugin::{
    validate_report_parts, StageEventHintV1, StageInvocationV1, StagePlugin, StagePluginOutputV1,
    StageReportPartV1,
};
pub use tool_binding::{
    canonical_external_artifact_tool_bindings, project_tool_parameters, render_tool_command,
    validate_effective_tool_parameters, validate_tool_binding, BindingInputV1, BindingOutputV1,
    BindingParameterV1, EffectiveToolParameterSourceV1, EffectiveToolParameterV1,
    EffectiveToolParametersV1, OutputDiscoveryV1, ToolArtifactSourceV1, ToolBinding,
    ToolBindingRegistry, ToolBindingRenderContext, ToolBindingV1, ToolParameterProjectionRequest,
    EFFECTIVE_TOOL_PARAMETERS_SCHEMA_V1, TOOL_BINDING_SCHEMA_V1,
};

pub use bijux_dna_core::contract::{
    artifact_compatibility, artifact_contract_catalog_schema, artifact_contract_schema,
    canonical_artifact_contract_catalog, ensure_artifact_compatibility, ArtifactCardinalityV1,
    ArtifactClassV1, ArtifactCompatibilityCheckV1, ArtifactCompatibilityReportV1,
    ArtifactCompressionV1, ArtifactContractCatalogV1, ArtifactContractV1, ArtifactFormatV1,
    ArtifactLineageContractV1, ArtifactPairingV1, ArtifactPresenceV1, ArtifactRef, ArtifactRole,
    ArtifactSidecarContractV1, ArtifactSidecarRoleV1, ArtifactSortOrderV1,
    ArtifactValidationInvariantV1, StageIO, ARTIFACT_CONTRACT_CATALOG_SCHEMA_V1,
    ARTIFACT_CONTRACT_SCHEMA_V1,
};
