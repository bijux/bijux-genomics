//! Contract bible for Bijux (stable, serialized interfaces).

pub mod canonical;
pub mod execution;
pub mod run;
/// Tooling contracts and selection semantics.
pub mod tooling;
/// Contract versioning and compatibility rules.
pub mod version;

pub use crate::ids::{
    ArtifactId, ImageDigest, PipelineId, ProfileId, RunId, StageId, StageVersion, StepId, ToolId,
    ToolVersion,
};
pub use execution::{
    validate_execution_outputs, ArtifactRef, ArtifactRole, ArtifactRoleFamily, ArtifactSpec,
    ExecutionEdge, ExecutionGraph, ExecutionManifest, ExecutionStep, PlanPolicy, RetryPolicy,
    RunRecordV1, StageExecutionRecordV1, StageIO,
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
