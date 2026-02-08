//! Metrics subsystem contracts and registries.

#![allow(missing_docs)]

mod registry;
mod semantics;
mod types;

pub use registry::{
    metrics_schema_for_stage, MetricsSchemaId, BAM_METRICS_SCHEMAS, FASTQ_METRICS_SCHEMAS,
};
pub use semantics::*;
pub use types::*;
