//! Adapter command acquisition preserves explicit family authority and pipeline semantics.
mod acquisition;
mod projection;
pub use acquisition::collect_vcf_adapter_command_row_map;

mod admission;
mod paths;
mod publication;
mod selection;
pub use publication::render_vcf_commands_with_observations;
pub use selection::collect_vcf_rendered_command_rows_with_observations;
pub const DEFAULT_VCF_RENDERED_COMMANDS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-rendered-commands.sh";
pub const VCF_RENDERED_COMMANDS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_rendered_commands.v1";
