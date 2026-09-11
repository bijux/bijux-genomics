//! VCF schema publication reports retain stable stage metadata.
mod model;
mod service;
pub use model::{VcfNormalizedMetricsSchemaReport, VcfNormalizedMetricsSchemaRow};
pub use service::{
    collect_vcf_normalized_metrics_schema_report_rows, render_vcf_normalized_metrics_schema,
};
