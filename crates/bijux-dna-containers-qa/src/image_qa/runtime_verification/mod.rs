//! Effectful catalog-wide Apptainer runtime verification.

mod apptainer_smoke;
mod workflow;

pub use apptainer_smoke::{
    can_reuse_apptainer_smoke, run_apptainer_smoke, ApptainerSmokeManifest,
    ApptainerSmokeProbeResult, ApptainerSmokeRequest, APPTAINER_SMOKE_MANIFEST_SCHEMA_V5,
    APPTAINER_SMOKE_MANIFEST_SCHEMA_V6,
};
pub use workflow::{
    execute_runtime_verification, RuntimeVerificationRequest, RuntimeVerificationResult,
};
