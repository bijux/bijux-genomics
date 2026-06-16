//! Metrics subsystem contracts and registries.

#![allow(missing_docs)]

mod registry;
mod semantics;
mod types;

pub use registry::{
    metrics_schema_for_stage, MetricsSchemaId, BAM_METRICS_SCHEMAS, FASTQ_METRICS_SCHEMAS,
};
pub use semantics::{metric_semantics, BankRefV1, MetricContextV1, MetricDirection, MetricSemantics};
pub use types::{
    parse_derived_metric_id, parse_metric_id, validate_derived_metric_id_str,
    validate_metric_id_str, AdapterBankProvenanceV1, BankEntryV1, DerivedMetricId, MetricEnvelope,
    MetricId, MetricSet, MetricsEnvelope, StageMetricsV1, ToolInvocationSpecV1, ToolInvocationV1,
};
