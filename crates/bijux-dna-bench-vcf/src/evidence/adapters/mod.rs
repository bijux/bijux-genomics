//! Adapter readiness over admitted inputs, shared observations and explicit syntax inspection.
mod checks;
mod model;
mod ports;
mod projection;
mod service;
mod state;
pub use model::{VcfAdaptersReadyGoalCheck, VcfAdaptersReadyReport};
pub use ports::{ShellSyntaxObservation, VcfAdapterReadinessPorts};
pub use service::render_vcf_adapters_ready;
pub const DEFAULT_VCF_ADAPTERS_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/VCF_ADAPTERS_READY.json";
pub const VCF_ADAPTERS_READY_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_adapters_ready.v1";

#[cfg(test)]
#[path = "../../../tests/contracts/adapter_readiness_projection.rs"]
mod projection_contract;
mod syntax;
#[cfg(test)]
#[path = "../../../tests/contracts/adapter_readiness_syntax.rs"]
mod syntax_contract;
