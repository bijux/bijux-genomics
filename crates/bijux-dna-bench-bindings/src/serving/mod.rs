//! Serving-map admission, projection and publication.
mod admission;
mod corpus;
mod labels;
mod models;
mod publication;
mod service;
mod tsv;
pub use corpus::load_corpus_status_by_stage;
pub use models::{ToolServingMapReport, ToolServingMapRow};
pub use publication::{render_bam_tool_serving_map, render_fastq_tool_serving_map};
pub use service::{collect, ServingRows};
pub const FASTQ_TOOL_SERVING_MAP_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_tool_serving_map.v1";
pub const BAM_TOOL_SERVING_MAP_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_tool_serving_map.v1";
