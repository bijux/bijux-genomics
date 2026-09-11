use crate::observations::ToolAdmission;
use anyhow::{anyhow, Result};
pub(super) fn admission_support_status<'a>(
    admissions: &'a [ToolAdmission],
    tool_id: &str,
) -> Result<&'a str> {
    admissions
        .iter()
        .find(|admission| admission.tool_id == tool_id)
        .map(|admission| admission.support_status.as_str())
        .ok_or_else(|| anyhow!("BAM report map stage is missing support status for `{tool_id}`"))
}
pub(super) fn select_anchor_tool<'a>(
    admissions: &'a [ToolAdmission],
    preferred_tool_id: &str,
) -> Result<(&'a str, &'a str)> {
    if let Some(admission) =
        admissions.iter().find(|admission| admission.tool_id == preferred_tool_id)
    {
        return Ok((admission.tool_id.as_str(), admission.support_status.as_str()));
    }

    let admission = admissions
        .iter()
        .min_by(|left, right| {
            admission_support_rank(&left.support_status)
                .cmp(&admission_support_rank(&right.support_status))
                .then_with(|| left.tool_id.cmp(&right.tool_id))
        })
        .ok_or_else(|| anyhow!("BAM report map stage has no admitted benchmark tools"))?;
    Ok((admission.tool_id.as_str(), admission.support_status.as_str()))
}
pub(super) fn admission_support_rank(status: &str) -> usize {
    match status {
        "supported" => 0,
        "planned" => 1,
        _ => 2,
    }
}
