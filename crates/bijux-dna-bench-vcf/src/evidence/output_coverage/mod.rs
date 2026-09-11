//! Declared VCF output coverage with typed observations and explicit publication.
mod acquisition;
mod authority;
mod contract;
mod observations;
mod projection;
mod publication;
pub use acquisition::collect_vcf_adapter_output_coverage_rows;
pub use publication::render_vcf_adapter_output_coverage;
pub const DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-adapter-output-coverage.tsv";
pub const VCF_ADAPTER_OUTPUT_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_adapter_output_coverage.v1";
