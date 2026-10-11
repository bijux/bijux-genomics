//! Governed qualification schemas derived from the owning policy types.

use super::{QualificationAssessment, QualificationReport};

/// Render the governed qualification-assessment schema.
#[must_use]
pub fn render_qualification_assessment_schema() -> serde_json::Value {
    schemars::schema_for!(QualificationAssessment).into()
}

/// Render the governed qualification-report schema.
#[must_use]
pub fn render_qualification_report_schema() -> serde_json::Value {
    schemars::schema_for!(QualificationReport).into()
}
