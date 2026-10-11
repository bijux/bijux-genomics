//! Report placement validates expected binding identities before atomic publication.
mod admission;
mod contract;
mod governed;
mod placement;
mod projection;
mod publication;
pub use bijux_dna_bench_vcf_evidence::model::reports::DEFAULT_VCF_REPORT_MAP_PATH;
pub use projection::collect_vcf_report_map_rows_from_expected;
pub use publication::render_vcf_report_map;
pub const VCF_REPORT_MAP_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_report_map.v1";
