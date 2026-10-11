//! Local submission readiness over explicit provider evidence and governed report contracts.
mod assessment;
mod campaign;
mod context;
mod corpora;
mod declarations;
mod harness;
mod paths;
mod pipelines;
pub mod ports;
mod service;
mod slurm;
mod stages;
mod watchdogs;
pub use context::{ReadinessContext, ReadinessInputPaths};
pub use declarations::DEFAULT_HPC_SUBMISSION_READY_REPORT_PATH;
pub use service::validate_hpc_submission_ready;
