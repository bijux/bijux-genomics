//! Stable public API facade for environment QA consumers.

pub mod api {
    pub use crate::runtime_verification::{
        execute_runtime_verification, RuntimeVerificationRequest, RuntimeVerificationResult,
    };
    pub use bijux_dna_containers::api::*;
}
