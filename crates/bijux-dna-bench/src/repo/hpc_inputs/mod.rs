//! Owner: HPC command-path interpretation and current source observations.
mod discovery;
mod observations;
mod path_usage;
pub use discovery::collect_hpc_job_source_inputs;
pub use observations::{HpcDiscoveredSourceInput, HpcStageInputHint};
pub use path_usage::materialize_stage_id;
#[cfg(test)]
mod tests;
