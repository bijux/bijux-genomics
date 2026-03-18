//! Runtime contracts and telemetry wiring.

pub mod environment;
pub mod manifests;
pub mod observability;
pub mod provenance;
pub mod recording;
pub mod run;
pub mod run_layout;
pub mod runner;
pub mod telemetry;

pub use observability::*;
pub use recording::{prepare_tool_run_dirs, write_canonical_json, write_run_manifest};
pub use run_layout::{create_run_layout, write_manifest, RunManifest, RunStageEntry};
pub use runner::{
    ensure_stage_supported_by_runner, Artifact, Invocation, Runner, RunnerContractKind,
    RunnerResult,
};
pub use telemetry::{build_telemetry_adapter, TelemetryAdapter, TelemetrySpan};
