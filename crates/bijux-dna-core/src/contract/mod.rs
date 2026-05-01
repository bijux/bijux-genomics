//! Contract bible for Bijux (stable, serialized interfaces).

pub mod canonical;
/// Governed schema compatibility and manifest migration rules.
pub mod compatibility;
pub mod execution;
/// Workflow and plan manifest contracts shared across planners and API surfaces.
pub mod planning;
pub mod run;
/// Tooling contracts and selection semantics.
pub mod tooling;
/// Contract versioning and compatibility rules.
pub mod version;

pub use crate::ids::{
    ArtifactId, ImageDigest, PipelineId, ProfileId, RunId, StageId, StageVersion, StepId, ToolId,
    ToolVersion,
};
pub use compatibility::{
    governed_api_route_adapters, governed_error_code_registry, governed_schema_registry,
    migrate_plan_manifest_value, migrate_workflow_manifest_value, schema_registry_entry,
    ApiRouteAdapterV1, ErrorRegistryEntryV1, GovernedErrorAreaV1, ManifestMigrationAuditV1,
    ManifestMigrationStatusV1, SchemaCompatibilityClassV1, SchemaMigrationRuleV1,
    SchemaRegistryEntryV1, SchemaSurfaceKindV1,
};
pub use execution::{
    validate_execution_outputs, ArtifactRef, ArtifactRole, ArtifactRoleFamily, ArtifactSpec,
    ExecutionEdge, ExecutionGraph, ExecutionManifest, ExecutionStep, PlanPolicy, RetryPolicy,
    RunRecordV1, StageExecutionRecordV1, StageIO,
};
pub use planning::{
    build_plan_manifest, diff_plan_manifests, planner_refusal_from_message,
    validate_cross_domain_handoffs, CrossDomainHandoffCheckV1, CrossDomainHandoffV1,
    ParameterResolutionTraceV1, PlanArtifactPromiseV1, PlanEnvironmentContractV1,
    PlanFieldChangeV1, PlanManifestBuildInputV1, PlanManifestDiffV1, PlanManifestStepV1,
    PlanManifestV1, PlannerParameterSourceV1, PlannerRefusalCodeV1, PlannerRefusalRecordV1,
    PlannerWarningCodeV1, PlannerWarningRecordV1, WorkflowEvidenceExpectationV1,
    WorkflowExecutorPreferencesV1, WorkflowInputArtifactV1, WorkflowManifestV1,
    WorkflowPolicySurfaceV1, WorkflowReferenceAssetV1, WorkflowStageDecisionKindV1,
    WorkflowStageDecisionV1, WorkflowStageRequestV1,
};
pub use run::{
    list_runs, query_latest_runs, query_run, query_runs, query_stage_rows, run_dir,
    MetricProvenanceV1, PipelineDomain, PipelineEdgeSpec, PipelineNodeSpec, PipelineSpec, Profile,
    RunIndexEntry, RunIndexLine, RunMetadataV1, RunQuery, RunSpec, ScientificProvenanceV1,
    StageIndexRow, StageMetadataV1, ToolExecutionMetadataV1, ToolInvocationMetadataV1,
    ToolProvenanceV1,
};
pub use tooling::{
    merged_capability_contract, objective_spec, select_stage, ArtifactKind, BackendVersionPolicy,
    BenchResultRecord, BenchResultStatus, CanonicalStageContractV1, CanonicalStageParametersV1,
    Cardinality, CompressionSupport, Disqualification, ExecutionContract, ImageRequirements,
    IndexRequirement, Objective, ObjectiveSpec, ObjectiveWeights, PathSpec, PortSpec,
    ReadLayoutMode, ReferenceRequirement, ReportSeverity, RuntimeScale, StageCapabilitySpec,
    StageEnvironmentRequirements, StageFamily, StageOperatingMode, StageParameterSpec,
    StageRefusalCode, StageReportContract, StageReportKind, StageSelection, StageSemanticKind,
    StageSpec, ToolConstraints, ToolExecutionSpecV1, ToolManifest, ToolRegistry, ToolRole,
    ToolScore, UnsupportedParameterCombination,
};
pub use version::ContractVersion;
