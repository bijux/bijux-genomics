//! Run-level contracts (profiles, manifests, provenance, metadata, indices).

#![allow(missing_docs)]

mod domain;
mod index;
mod metadata;
mod provenance;
mod spec;

pub use domain::{PipelineDomain, PipelineSpec};
pub use index::{
    list_runs, query_latest_runs, query_run, query_runs, query_stage_rows, RunIndexEntry,
    RunIndexLine, RunQuery, StageIndexRow,
};
pub use metadata::{
    RunMetadataV1, StageMetadataV1, ToolExecutionMetadataV1, ToolInvocationMetadataV1,
};
pub use provenance::{MetricProvenanceV1, ScientificProvenanceV1, ToolProvenanceV1};
pub use spec::{run_dir, Profile, RunSpec};
