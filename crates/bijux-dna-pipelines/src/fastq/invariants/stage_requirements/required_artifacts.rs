use crate::InvariantSeverity;

use super::super::{violation, FastqProfileViolation};
use crate::PipelineProfile;

pub(super) fn push(profile: &PipelineProfile, violations: &mut Vec<FastqProfileViolation>) {
    for artifact in
        ["report.json", "run_manifest.json", "stage_summaries.json", "invariants_report.json"]
    {
        if !profile.capabilities.required_artifacts.contains(&artifact) {
            violations.push(violation(
                "required_artifact_missing",
                None,
                InvariantSeverity::Hard,
                format!("FASTQ profiles must require `{artifact}`"),
            ));
        }
    }
}
