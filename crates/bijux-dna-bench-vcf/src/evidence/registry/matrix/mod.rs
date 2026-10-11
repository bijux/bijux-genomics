//! Matrix registry coverage policy and compatible reports.
mod model;
mod policy;
mod projection;
mod service;
pub use model::{VcfMatrixRegistryConsistencyReport, VcfMatrixRegistryConsistencyRow};
pub use service::render_vcf_matrix_registry_consistency;
pub const DEFAULT_VCF_MATRIX_REGISTRY_CONSISTENCY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-matrix-registry-consistency.json";
pub const VCF_MATRIX_REGISTRY_CONSISTENCY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_matrix_registry_consistency.v1";
