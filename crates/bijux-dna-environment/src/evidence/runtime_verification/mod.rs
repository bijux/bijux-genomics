//! Catalog-wide runtime verification for canonical Apptainer images.

mod report;
mod workflow;

pub use report::{render_runtime_verification_markdown, verify_runtime_verification_report};
pub use workflow::{
    execute_runtime_verification, RuntimeVerificationRequest, RuntimeVerificationResult,
};
