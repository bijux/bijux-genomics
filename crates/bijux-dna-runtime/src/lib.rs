//! Runtime contracts and telemetry wiring.
#![allow(
    clippy::default_trait_access,
    clippy::expect_used,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::needless_pass_by_value,
    clippy::redundant_closure_for_method_calls,
    clippy::unnecessary_lazy_evaluations,
    clippy::uninlined_format_args
)]

pub mod environment;
pub mod manifests;
pub mod observability;
pub mod provenance;
pub mod recording;
pub mod run;
pub mod run_layout;
pub mod runner;
pub mod telemetry;

// Observability contracts
pub use observability::{
    attrs_from_json, redact_key, redacted_attrs, validate_stage_telemetry, AssetsProvenanceV1,
    AttrMap, AttrValue, EffectiveConfigV1, FactsRowV1, FailureCode, FilterReportV1, MergeReportV1,
    MetricSemanticsV1, PipelineVerdictV1, QcPostReportV1, ReportCompletenessV1, ReportContractV1,
    ReportProvenanceV1, ReportSchemaV1, ReportStageSummaryV1, RetentionContextV1,
    RetentionDefinitionV1, RetentionReportV1, RunContextV1, RunProvenanceV1,
    StageObservabilityContextV1, StageObservabilityContractV1, StageReportV1, TelemetryEventName,
    TelemetryEventV1, TrimReportV1, ValidateReportV1,
};

// Recording entrypoints
pub use recording::{
    append_attempt_lifecycle_transition, benchmark_attempt_directory, ensure_attempt_lifecycle,
    initialize_attempt_lifecycle, load_attempt_lifecycle, load_attempt_scheduler_observations,
    load_run_evidence_index, prepare_tool_run_dirs, record_attempt_acceptance,
    record_attempt_accounting, record_attempt_blocked_external, record_attempt_ready,
    record_attempt_running, record_attempt_scheduler_observation, record_attempt_submission,
    record_attempt_superseded, validate_run_evidence_index, write_canonical_json,
    write_profile_and_lock_manifests, write_run_manifest, AttemptExecutionStatus, RunEvidenceIndex,
    RunEvidenceRecord, ScientificAcceptanceStatus, CONTAINER_RUN_EVIDENCE_INDEX_SCHEMA_V2,
};

// Run layout entrypoints
pub use run_layout::{
    create_run_layout, write_checkpoint, write_executor_descriptor, write_failure_record,
    write_manifest, write_run_state, write_runtime_policy, RunManifest, RunStageEntry,
};

// Runner contracts and execution models
pub use runner::{
    ensure_stage_supported_by_runner, Artifact, Invocation, Runner, RunnerContractKind,
    RunnerResult,
};

// Runtime telemetry adapter
pub use telemetry::{build_telemetry_adapter, TelemetryAdapter, TelemetrySpan};
