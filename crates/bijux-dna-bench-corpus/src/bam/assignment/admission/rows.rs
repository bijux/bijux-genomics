use crate::bam::assignment::models::BamCorpusAssignmentRow;
use anyhow::anyhow;
use anyhow::Result;
use std::collections::BTreeSet;
pub(in crate::bam::assignment) fn ensure_row_completeness(
    rows: &[BamCorpusAssignmentRow],
) -> Result<()> {
    let mut seen = BTreeSet::<(&str, &str)>::new();
    for row in rows {
        if !seen.insert((&row.stage_id, &row.tool_id)) {
            return Err(anyhow!(
                "BAM corpus assignment report repeats row `{}` / `{}`",
                row.stage_id,
                row.tool_id
            ));
        }
    }
    Ok(())
}
