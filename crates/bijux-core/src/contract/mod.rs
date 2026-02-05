pub mod domain;
pub mod execution_contract;
pub mod execution_manifest;
pub mod metadata;
pub mod provenance;
pub mod run;
pub mod run_record;
pub mod selection;
pub mod tooling;

pub use domain::{PipelineDomain, PipelineSpec};
pub use execution_contract::validate_execution_outputs;
pub use execution_manifest::ExecutionManifest;
pub use metadata::{
    RunMetadataV1, StageMetadataV1, ToolExecutionMetadataV1, ToolInvocationMetadataV1,
};
pub use provenance::{ScientificProvenanceV1, ToolProvenanceV1};
pub use run::{
    build_run_execution_plan, run_dir, DryRunExecutor, Executor, Profile, RunExecutionPlan, RunId,
    RunSpec,
};
pub use run_record::{RunRecordV1, StageExecutionRecordV1};
pub use selection::{
    BenchResultRecord, BenchResultStatus, Disqualification, Objective, ObjectiveSpec,
    ObjectiveWeights, StageSelection, ToolScore,
};
pub use tooling::{
    Cardinality, ExecutionContract, PathSpec, PortSpec, StageId, StageSpec, StageVersion,
    ToolConstraints, ToolExecutionSpecV1, ToolId, ToolManifest, ToolRegistry, ToolRole,
};
