//! Exact-image security probe execution and raw evidence publication.

mod workflow;

pub use workflow::{
    execute_security_verification, SecurityVerificationRequest, SecurityVerificationResult,
};
