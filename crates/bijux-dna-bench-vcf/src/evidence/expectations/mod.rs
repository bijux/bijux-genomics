//! Expected VCF result declarations over admitted bindings and request observations.
mod contract;
mod governed;
mod outputs;
mod projection;
mod publication;
pub use projection::collect_vcf_expected_benchmark_result_rows;
pub use publication::render_vcf_expected_benchmark_results;
pub const DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-expected-benchmark-results.tsv";
pub const VCF_EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_expected_benchmark_results.v1";
