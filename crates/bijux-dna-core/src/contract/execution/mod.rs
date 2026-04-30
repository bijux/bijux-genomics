//! Execution-level contracts (graph, policy, IO, manifests, records).

#![allow(missing_docs)]

mod contract;
mod graph;
mod io;
mod manifest;
mod policy;
mod record;

pub use contract::validate_execution_outputs;
pub use graph::{ExecutionEdge, ExecutionGraph, ExecutionStep};
pub use io::{ArtifactRef, ArtifactRole, ArtifactRoleFamily, ArtifactSpec, StageIO};
pub use manifest::ExecutionManifest;
pub use policy::{PlanPolicy, RetryPolicy};
pub use record::{RunRecordV1, StageExecutionRecordV1};
