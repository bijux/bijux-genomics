//! Parser-readiness orchestration preserves each independent diagnostic check.
mod checks;
mod fixtures;
mod model;
mod projection;
mod service;
mod state;
pub use model::{VcfParsersReportReadyGoalCheck, VcfParsersReportReadyReport};
pub use service::{render_vcf_parsers_report_ready, ActiveBindingProvider};
pub const DEFAULT_VCF_PARSERS_REPORT_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/VCF_PARSERS_REPORT_READY.json";
pub const VCF_PARSERS_REPORT_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_parsers_report_ready.v1";

#[cfg(test)]
#[path = "../../../../tests/contracts/parser_readiness_projection.rs"]
mod projection_contract;

#[cfg(test)]
#[path = "../../../../tests/contracts/parser_pair_completeness.rs"]
mod pair_contract;
