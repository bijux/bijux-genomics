mod call;
mod effective;
mod filter;
mod stats;

pub use call::VcfCallParams;
pub use effective::VcfEffectiveParams;
pub use filter::VcfFilterParams;
pub use stats::VcfStatsParams;

pub const PARAM_SCHEMA_V1: &str = "bijux.vcf.params.v1";
