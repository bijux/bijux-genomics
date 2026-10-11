//! Configured HPC campaigns and their local planning reports.
mod bundle;
mod configuration;
mod reports;
pub use configuration::*;
pub use reports::*;

pub use bundle::{sidecar_path_for, BundleSidecar, BUNDLE_SIDECAR_SCHEMA_VERSION};

mod monitor;
pub use monitor::{
    SlurmMonitorEntry, SlurmMonitorReport, SlurmMonitorSnapshot, CAMPAIGN_MONITOR_SCHEMA_VERSION,
};

mod copy_back;
pub use copy_back::{CopyBackEntry, CopyBackManifestReport, COPY_BACK_MANIFEST_SCHEMA_VERSION};

mod submission_bundle;
pub use submission_bundle::{
    assemble_hpc_code_submission, assemble_hpc_results_submission, HpcBundleFoundationLocks,
    HpcBundleSubmission, HpcCodeFreezeObservation, HpcCodeSubmissionBundle,
    HpcResultsSubmissionBundle,
};

mod submission_report;
pub use submission_report::{
    validate_hpc_scheduler_receipt, validate_hpc_submission_report, HpcSubmissionMode,
    HpcSubmissionSettings, SlurmSubmissionReport, SubmittedJob,
    HPC_SLURM_SUBMISSION_SCHEMA_VERSION,
};
