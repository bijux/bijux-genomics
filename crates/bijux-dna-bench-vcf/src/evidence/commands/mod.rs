//! Adapter command acquisition preserves explicit family authority and pipeline semantics.
mod acquisition;
mod projection;
pub use acquisition::collect_vcf_adapter_command_row_map;
