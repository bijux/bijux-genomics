use crate::InvariantSeverity;

use super::FastqProfileViolation;

pub(super) fn violation(
    code: &'static str,
    stage_id: Option<&str>,
    severity: InvariantSeverity,
    message: impl Into<String>,
) -> FastqProfileViolation {
    FastqProfileViolation {
        code,
        stage_id: stage_id.map(str::to_string),
        severity,
        message: message.into(),
    }
}
