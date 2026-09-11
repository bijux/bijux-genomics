use super::declarations::FASTQ_VALIDATE_READS_STAGE_ID;
use super::declarations::REQUIRED_TOOL_IDS;
use super::declarations::REQUIRED_VALIDATED_READS_MANIFEST_FIELDS;
use super::declarations::REQUIRED_VALIDATION_REPORT_FIELDS;
use crate::proofs::inputs::observations::CommandArgvObservation as RenderedCommandArgvRow;
use anyhow::anyhow;
use anyhow::Result;
use std::collections::BTreeSet;
pub(super) fn command_has_required_validation_fields(
    row: &RenderedCommandArgvRow,
    tool_id: &str,
) -> bool {
    let argv_text = row.argv.join(" ");
    argv_text.contains(&format!("\"stage_id\":\"{FASTQ_VALIDATE_READS_STAGE_ID}\""))
        && argv_text.contains(&format!("\"tool_id\":\"{tool_id}\""))
        && argv_text.contains("validation.json")
        && argv_text.contains("validated_reads_manifest.json")
        && required_validation_report_fields()
            .iter()
            .all(|field| argv_text.contains(&format!("\"{field}\"")))
        && required_validated_reads_manifest_fields()
            .iter()
            .all(|field| argv_text.contains(&format!("\"{field}\"")))
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
