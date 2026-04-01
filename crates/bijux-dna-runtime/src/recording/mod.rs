//! Recording and runtime emit helpers.
//!
//! Boundaries:
//! - Only write under the run layout.
//! - No heavy dependencies; keep this module lightweight and stable.

mod envelope;
mod io;
mod manifests;
mod metrics;
mod provenance;
mod telemetry;

pub use envelope::{write_run_artifact_envelope, RunArtifactEnvelopeV1, StageResultStatus};
pub use io::{
    append_jsonl_line, hash_file_sha256, write_artifact_checksums_json, write_atomic_bytes,
    write_canonical_json, write_execution_logs, write_execution_logs_bounded,
};
pub use manifests::{
    compute_run_id, prepare_tool_run_dirs, run_artifacts_dir_for_out, tool_run_artifacts_dir,
    write_run_manifest, write_stage_plan_json, ObservabilityManifestV1, PlanArtifacts,
    ProgressEventV1, RunArtifactInput, RunDirs, RunsExportRowV1,
};
pub use metrics::{
    write_metrics_envelope, write_metrics_json, write_stage_metrics_json,
    write_tool_invocation_json,
};
pub use provenance::{write_plan_provenance, write_scientific_provenance};
pub use telemetry::write_telemetry_event;
