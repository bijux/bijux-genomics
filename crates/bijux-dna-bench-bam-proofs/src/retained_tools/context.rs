use super::ports::{BindingEvidence, QualificationEvidence, StageProofs};
use std::path::Path;
/// Canonical published evidence locations attached to aggregate diagnostics.
pub struct EvidencePaths<'a> {
    pub local_smoke: &'a str,
    pub parser_coverage: &'a str,
    pub rendered_commands: &'a str,
    pub output_declarations: &'a str,
    pub expected_results: &'a str,
    pub report_map: &'a str,
    pub active_row_consistency: &'a str,
    pub micro_smoke_summary: &'a str,
    pub science_thresholds: &'a str,
}
/// Explicit canonical sources; no global repository discovery or process mutation.
pub struct RetainedContext<'a> {
    pub repo_root: &'a Path,
    pub bindings: &'a dyn BindingEvidence,
    pub qualification: &'a dyn QualificationEvidence,
    pub stages: &'a dyn StageProofs,
    pub paths: EvidencePaths<'a>,
}
