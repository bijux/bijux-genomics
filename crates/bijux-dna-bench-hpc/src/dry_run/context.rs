use super::ports::{DryRunEvidenceProvider, DryRunOutputPolicy};
use std::path::Path;
/// Canonical provider destinations used when acquisition fails before returning a report.
pub struct DryRunInputPaths<'a> {
    pub asset: &'a str,
    pub scratch: &'a str,
    pub execution: &'a str,
    pub stage_benchmark: &'a str,
    pub pipeline_node: &'a str,
    pub dependency: &'a str,
    pub resume: &'a str,
    pub result_collection: &'a str,
    pub candidate: &'a str,
}
/// Request-scoped dependencies; no working-directory or environment discovery occurs here.
pub struct DryRunContext<'a> {
    pub repo_root: &'a Path,
    pub evidence: &'a dyn DryRunEvidenceProvider,
    pub output: &'a dyn DryRunOutputPolicy,
    pub paths: DryRunInputPaths<'a>,
}
