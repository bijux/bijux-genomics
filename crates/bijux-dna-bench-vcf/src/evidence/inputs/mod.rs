//! Required-input audit from admitted adapters and shared file validation.
mod acquisition;
mod coverage;
mod model;
mod probes;
mod service;
pub use model::{VcfAdapterMissingInputTestRow, VcfAdapterMissingInputTestsReport};
pub use service::{
    render_vcf_adapter_missing_input_audit,
    render_vcf_adapter_missing_input_audit_with_observations,
};
pub const DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-adapter-missing-input-tests.json";
pub const VCF_ADAPTER_MISSING_INPUT_TESTS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_adapter_missing_input_audit.v1";
