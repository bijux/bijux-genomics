use super::ports::{CommandEvidence, OutputEvidence, ReportEvidence};
use std::path::Path;
/// Output contracts bind source evidence and canonical run-path identity to an explicit root.
pub struct OutputContext<'a> {
    pub repo_root: &'a Path,
    pub evidence: &'a dyn OutputEvidence,
    pub run_id: &'a str,
}
/// Reporting acquires only expected rows, inventory and support admission.
pub struct ReportContext<'a> {
    pub repo_root: &'a Path,
    pub evidence: &'a dyn ReportEvidence,
}
/// Command projection acquires only active bindings and their structured execution declarations.
pub struct CommandContext<'a> {
    pub repo_root: &'a Path,
    pub evidence: &'a dyn CommandEvidence,
}
