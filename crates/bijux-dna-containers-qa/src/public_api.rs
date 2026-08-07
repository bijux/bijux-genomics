//! Stable public API facade for environment QA consumers.

pub mod api {
    pub use crate::qualification::{
        evaluate_container_qualification, plan_container_qualification,
        ContainerQualificationAssessment, ContainerQualificationPlan, QualificationFailure,
        QualificationFailureClass, QualificationInputBind, QualificationMetricEvidence,
        QualificationMetricPlan, QualificationOutputEvidence, QualificationProbeKind,
        QualificationProbeObservation, QualificationProbePlan,
    };
    pub use crate::runtime_verification::{
        execute_runtime_verification, RuntimeVerificationRequest, RuntimeVerificationResult,
    };
    pub use bijux_dna_containers::api::*;
}
