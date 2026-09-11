//! Governed HPC dry-run assessment, canonical report publication and fresh replay.
mod assessment;
mod behavior;
mod checks;
mod context;
mod evidence;
pub mod ports;
mod publication;
mod summary;
pub use context::{DryRunContext, DryRunInputPaths};
pub use publication::{render_hpc_dry_run_ready, validate_hpc_dry_run_ready_path};
