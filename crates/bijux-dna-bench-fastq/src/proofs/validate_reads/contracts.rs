use super::declarations::FASTQ_VALIDATE_READS_STAGE_ID;
use super::declarations::REQUIRED_TOOL_IDS;
use super::declarations::REQUIRED_VALIDATED_READS_MANIFEST_FIELDS;
use super::declarations::REQUIRED_VALIDATION_REPORT_FIELDS;
use crate::proofs::inputs::observations::CommandArgvObservation as RenderedCommandArgvRow;
use anyhow::anyhow;
use anyhow::Result;
use std::collections::BTreeSet;
pub(super) fn command_is_admitted(
    context: &crate::proofs::inputs::ValidationContext<'_>,
    row: &RenderedCommandArgvRow,
    tool_id: &str,
) -> bool {
    row.stage_id == FASTQ_VALIDATE_READS_STAGE_ID
        && row.tool_id == tool_id
        && context.validation.validate_command(context.repo_root, row).is_ok()
}

pub(super) fn ensure_required_tool_set<'a>(
    surface: &str,
    tool_ids: impl Iterator<Item = &'a str>,
) -> Result<()> {
    let observed = tool_ids.map(str::to_string).collect::<BTreeSet<_>>();
    let expected = required_tool_ids().into_iter().collect::<BTreeSet<_>>();
    if observed != expected {
        return Err(anyhow!("{surface} must contain exactly {expected:?}, found {observed:?}"));
    }
    Ok(())
}

pub(super) fn required_tool_ids() -> Vec<String> {
    REQUIRED_TOOL_IDS.iter().map(|value| (*value).to_string()).collect()
}

pub(super) fn required_validation_report_fields() -> Vec<String> {
    REQUIRED_VALIDATION_REPORT_FIELDS.iter().map(|value| (*value).to_string()).collect()
}

pub(super) fn required_validated_reads_manifest_fields() -> Vec<String> {
    REQUIRED_VALIDATED_READS_MANIFEST_FIELDS.iter().map(|value| (*value).to_string()).collect()
}
