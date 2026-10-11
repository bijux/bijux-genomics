//! Stable VCF smoke evidence contracts exposed through the v1 benchmark API.

pub use bijux_dna_bench::{
    benchmark_local_vcf_call_gl_smoke_metrics_schema, benchmark_local_vcf_call_gl_smoke_schema,
    validate_benchmark_local_vcf_call_gl_smoke_metrics,
    validate_benchmark_local_vcf_call_gl_smoke_report, BenchmarkLocalVcfCallGlSmokeMetrics,
    BenchmarkLocalVcfCallGlSmokeReport, BENCHMARK_LOCAL_VCF_CALL_GL_SMOKE_METRICS_SCHEMA_V1,
    BENCHMARK_LOCAL_VCF_CALL_GL_SMOKE_SCHEMA_V1,
};
pub use bijux_dna_bench::{
    benchmark_local_vcf_call_smoke_metrics_schema, benchmark_local_vcf_call_smoke_schema,
    validate_benchmark_local_vcf_call_smoke_metrics,
    validate_benchmark_local_vcf_call_smoke_report, BenchmarkLocalVcfCallSmokeMetrics,
    BenchmarkLocalVcfCallSmokeReport, BENCHMARK_LOCAL_VCF_CALL_SMOKE_METRICS_SCHEMA_V1,
    BENCHMARK_LOCAL_VCF_CALL_SMOKE_SCHEMA_V1,
};
pub use bijux_dna_bench::{
    benchmark_local_vcf_ibd_smoke_schema, validate_benchmark_local_vcf_ibd_smoke_report,
    BenchmarkLocalVcfIbdInsufficientOverlapProbe, BenchmarkLocalVcfIbdSmokeReport,
    BenchmarkLocalVcfIbdSmokeRow, BENCHMARK_LOCAL_VCF_IBD_SMOKE_SCHEMA_V1,
};
pub use bijux_dna_bench::{
    benchmark_local_vcf_imputation_metrics_smoke_schema,
    validate_benchmark_local_vcf_imputation_metrics_smoke_report,
    BenchmarkImputationQualityFieldAvailability, BenchmarkLocalVcfImputationMetricsSmokeReport,
    BENCHMARK_LOCAL_VCF_IMPUTATION_METRICS_SMOKE_SCHEMA_V1,
};
