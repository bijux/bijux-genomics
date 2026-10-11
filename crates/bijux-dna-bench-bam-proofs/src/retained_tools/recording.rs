use super::models::BamAllRetainedToolsCompleteGoalCheck;
use anyhow::Result;
pub(super) fn record_goal_check<F>(
    checks: &mut Vec<BamAllRetainedToolsCompleteGoalCheck>,
    goal_id: u32,
    surface: impl Into<String>,
    output_path: Option<String>,
    check: F,
) where
    F: FnOnce() -> Result<String>,
{
    match check() {
        Ok(detail) => checks.push(BamAllRetainedToolsCompleteGoalCheck {
            goal_id,
            surface: surface.into(),
            output_path,
            ok: true,
            detail,
        }),
        Err(error) => checks.push(BamAllRetainedToolsCompleteGoalCheck {
            goal_id,
            surface: surface.into(),
            output_path,
            ok: false,
            detail: error.to_string(),
        }),
    }
}
