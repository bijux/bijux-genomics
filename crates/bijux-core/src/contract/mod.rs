mod domain;
mod execution_contract;
mod execution_manifest;
mod io;
mod provenance;
mod run;
mod run_record;
mod selection;
mod tooling;
mod version;

pub use crate::ids::{
    ArtifactId, ImageDigest, PipelineId, ProfileId, RunId, StageId, StageVersion, StepId, ToolId,
    ToolVersion,
};
pub use crate::metadata::{
    RunMetadataV1, StageMetadataV1, ToolExecutionMetadataV1, ToolInvocationMetadataV1,
};
pub use domain::{PipelineDomain, PipelineSpec};
pub use execution_contract::validate_execution_outputs;
pub use execution_manifest::ExecutionManifest;
pub use io::{ArtifactRef, ArtifactRole, ArtifactSpec, StageIO};
pub use provenance::{ScientificProvenanceV1, ToolProvenanceV1};
pub use run::{run_dir, Profile, RunSpec};
pub use run_record::{RunRecordV1, StageExecutionRecordV1};
pub use selection::{
    BenchResultRecord, BenchResultStatus, Disqualification, Objective, ObjectiveSpec,
    ObjectiveWeights, StageSelection, ToolScore,
};
pub use tooling::{
    Cardinality, ExecutionContract, PathSpec, PortSpec, StageSpec, ToolConstraints,
    ToolExecutionSpecV1, ToolManifest, ToolRegistry, ToolRole,
};
pub use version::ContractVersion;
