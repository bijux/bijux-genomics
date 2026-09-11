//! Comparable metric declarations preserve domain units and direction semantics.
mod acquisition;
mod projection;
mod serialization;
mod service;
pub use bijux_dna_bench_model::{
    ComparableMetricDirection, VcfComparableMetricsReport, VcfComparableMetricsRow,
    VCF_COMPARABLE_METRICS_SCHEMA_V1,
};
pub use service::render_vcf_comparable_metrics;
pub const DEFAULT_VCF_COMPARABLE_METRICS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-comparable-metrics.tsv";
#[cfg(test)]
#[path = "../../../tests/contracts/comparable_metric_direction.rs"]
mod direction_contract;
