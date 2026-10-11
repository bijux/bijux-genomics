//! QA-owned container qualification planning and verdict evaluation.

mod evaluation;
mod model;
mod planning;

pub use evaluation::evaluate_container_qualification;
pub use model::{
    ContainerQualificationAssessment, ContainerQualificationPlan, QualificationFailure,
    QualificationFailureClass, QualificationInputBind, QualificationMetricEvidence,
    QualificationMetricPlan, QualificationOutputEvidence, QualificationProbeKind,
    QualificationProbeObservation, QualificationProbePlan,
};
pub use planning::plan_container_qualification;
