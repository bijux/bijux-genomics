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
    validate_execution_outputs, ArtifactRef, ArtifactRole, ArtifactSpec, ExecutionEdge,
    ExecutionGraph, ExecutionManifest, ExecutionStep, PlanPolicy, RetryPolicy, RunRecordV1,
    StageExecutionRecordV1, StageIO,
};
pub use run::{
    list_runs, query_latest_runs, query_run, query_runs, query_stage_rows, run_dir, PipelineDomain,
    PipelineSpec, Profile, RunIndexEntry, RunIndexLine, RunMetadataV1, RunQuery, RunSpec,
    MetricProvenanceV1, ScientificProvenanceV1, StageIndexRow, StageMetadataV1, ToolExecutionMetadataV1,
    ToolInvocationMetadataV1, ToolProvenanceV1,
};
pub use tooling::{
    objective_spec, select_stage, BenchResultRecord, BenchResultStatus, Cardinality,
    Disqualification, ExecutionContract, ImageRequirements, Objective, ObjectiveSpec,
    ObjectiveWeights, PathSpec, PortSpec, StageParameterSpec, StageSelection, StageSpec,
    ToolConstraints, ToolExecutionSpecV1, ToolManifest, ToolRegistry, ToolRole, ToolScore,
};
pub use version::ContractVersion;
