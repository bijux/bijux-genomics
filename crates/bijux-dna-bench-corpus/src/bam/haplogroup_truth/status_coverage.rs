//! Require the fixture bundle to retain every supported haplogroup outcome class.
use super::model::BamHaplogroupCaseTruth;
use anyhow::{anyhow, Result};
use std::path::Path;

pub(super) fn validate_required_statuses(statuses: &[String], path: &Path) -> Result<()> {
    let required = ["coverage_gate_not_met", "ready", "uncertain_marker_support"];
    for status in required {
        if !statuses.iter().any(|value| value == status) {
            return Err(anyhow!(
                "BAM haplogroup truth `{}` must cover the `{status}` status",
                path.display()
            ));
        }
    }
    Ok(())
}

pub(super) fn collect_statuses(case_truths: &[BamHaplogroupCaseTruth]) -> Vec<String> {
    let mut statuses =
        case_truths.iter().map(|case| case.summary.status.clone()).collect::<Vec<_>>();
    statuses.sort();
    statuses.dedup();
    statuses
}
