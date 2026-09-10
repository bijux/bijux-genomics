//! Submission payload contracts, pure assembly and explicit adapter inputs.
mod assembly;
mod inputs;
mod projections;
mod wire;
pub use assembly::{assemble_hpc_code_submission, assemble_hpc_results_submission};
pub use inputs::{HpcBundleFoundationLocks, HpcBundleSubmission, HpcCodeFreezeObservation};
pub use wire::{HpcCodeSubmissionBundle, HpcResultsSubmissionBundle};
