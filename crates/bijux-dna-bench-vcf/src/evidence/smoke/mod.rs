//! Local smoke-path declarations distinguish available wrappers from execution evidence.
mod admission;
mod contract;
mod inputs;
mod model;
mod paths;
mod projection;
mod runtime;
mod serialization;
mod service;
pub use inputs::VcfSmokeInputs;
pub use model::{VcfLocalContainerSmokeReport, VcfLocalContainerSmokeRow};
pub use projection::collect_vcf_local_container_smoke_rows;
pub use service::render_vcf_local_container_smoke;
pub const DEFAULT_VCF_LOCAL_CONTAINER_SMOKE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-local-container-smoke.tsv";
pub const VCF_LOCAL_CONTAINER_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_local_container_smoke.v1";

mod rows;

mod governed;
