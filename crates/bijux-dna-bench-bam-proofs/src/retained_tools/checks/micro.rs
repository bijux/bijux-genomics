use super::super::acquisition::EvidenceSet;
use super::super::declarations::EXPECTED_MICRO_SMOKE_FAMILY_COUNT;
use anyhow::{bail, Result};
pub(in crate::retained_tools) fn check_micro(evidence: &EvidenceSet) -> Result<String> {
    if evidence.micro_smoke_summary.family_count != EXPECTED_MICRO_SMOKE_FAMILY_COUNT
        || !evidence.micro_smoke_summary.passes_behavior_test
    {
        bail!(
            "BAM micro smoke subset drifted: family_count={}, passes_behavior_test={}",
            evidence.micro_smoke_summary.family_count,
            evidence.micro_smoke_summary.passes_behavior_test
        );
    }
    Ok(format!(
                "validated one representative retained BAM micro smoke per stage family with {} host-local row(s), {} container-needed row(s), and {} unavailable row(s)",
                evidence.micro_smoke_summary.local_smoke_count,
                evidence.micro_smoke_summary.container_needed_count,
                evidence.micro_smoke_summary.unavailable_count
            ))
}
