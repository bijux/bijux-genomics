//! Undercoverage registry coverage policy and compatible reports.
mod model;
mod policy;
mod projection;
mod serialization;
mod service;
pub use model::{VcfUndercoveredStageRow, VcfUndercoveredStagesReport};
pub use service::render_vcf_undercovered_stages;
pub const DEFAULT_VCF_UNDERCOVERED_STAGES_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-undercovered-stages.tsv";
pub const VCF_UNDERCOVERED_STAGES_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_undercovered_stages.v1";
