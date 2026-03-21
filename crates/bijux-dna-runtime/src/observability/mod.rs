//! Observability and reporting contracts.
//!
//! Boundaries:
//! - Schema definitions only (no IO, no network).
//! - No heavy dependencies; keep this module lightweight and stable.

mod contracts;
mod reports;
mod telemetry;

pub use contracts::{EffectiveConfigV1, StageObservabilityContextV1, StageObservabilityContractV1};
pub use reports::{
    AssetsProvenanceV1, FilterReportV1, MergeReportV1, MetricSemanticsV1, PipelineVerdictV1,
    QcPostReportV1, ReportCompletenessV1, ReportContractV1, ReportProvenanceV1, ReportSchemaV1,
    ReportStageSummaryV1, RetentionContextV1, RetentionDefinitionV1, RetentionReportV1,
    StageReportV1, TrimReportV1, ValidateReportV1,
};
pub use telemetry::{
    attrs_from_json, redact_key, redacted_attrs, validate_stage_telemetry, AttrMap, AttrValue,
    FactsRowV1, FailureCode, RunContextV1, RunProvenanceV1, TelemetryEventName, TelemetryEventV1,
};
