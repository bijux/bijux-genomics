//! Serving observations derived from admitted VCF catalog and supported bindings.
mod contract;
mod model;
mod projection;
mod publication;
pub use model::{VcfToolServingMapReport, VcfToolServingMapRow};
pub use projection::collect_vcf_tool_serving_map_rows;
pub use publication::render_vcf_tool_serving_map;
pub const DEFAULT_VCF_TOOL_SERVING_MAP_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-tool-serving-map.tsv";
pub const VCF_TOOL_SERVING_MAP_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_tool_serving_map.v1";
