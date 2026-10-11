//! Governed local candidate eligibility, representative selection and replay.
mod behavior;
mod context;
mod declarations;
mod eligibility;
mod inputs;
mod paths;
pub mod ports;
mod projection;
mod publication;
mod resources;
mod selection;
mod service;
pub use context::CandidateContext;
pub use publication::{
    render_hpc_candidate_run_manifest, validate_hpc_candidate_run_manifest_path,
};
pub use service::collect_hpc_candidate_run_manifest;
