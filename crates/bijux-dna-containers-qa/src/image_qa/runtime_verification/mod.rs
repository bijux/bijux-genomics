//! Effectful catalog-wide Apptainer runtime verification.

mod workflow;

pub use workflow::{
    execute_runtime_verification, RuntimeVerificationRequest, RuntimeVerificationResult,
};
