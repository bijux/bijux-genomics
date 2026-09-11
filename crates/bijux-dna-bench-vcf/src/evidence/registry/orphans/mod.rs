//! Orphans registry coverage policy and compatible reports.
mod model;
mod policy;
mod projection;
mod serialization;
mod service;
pub use model::{VcfOrphanToolRow, VcfOrphanToolsReport};
pub use service::render_vcf_orphan_tools;
pub const DEFAULT_VCF_ORPHAN_TOOLS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-orphan-tools.tsv";
pub const VCF_ORPHAN_TOOLS_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_orphan_tools.v1";
