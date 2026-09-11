//! Expected VCF result declarations over admitted bindings and request observations.
mod contract;
mod governed;
mod outputs;
mod projection;
mod publication;
pub use bijux_dna_bench_vcf_evidence::model::expectations::DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH;
pub use projection::collect_vcf_expected_benchmark_result_rows;
pub use publication::render_vcf_expected_benchmark_results;
pub const VCF_EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_expected_benchmark_results.v1";
