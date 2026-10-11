//! Shared typed registry admission and independent coverage audits.
mod acquisition;
mod admission;
mod declarations;
pub use acquisition::load_vcf_registry_snapshot;
pub use admission::VcfRegistrySnapshot;
pub mod matrix;
pub mod orphans;
pub mod undercoverage;
