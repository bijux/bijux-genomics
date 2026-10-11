use crate::ports::{ConsistencyEvidence, OutputEvidence, ReportEvidence, ServingEvidence};
use std::path::Path;
/// Canonical publication defaults supplied by the CLI composition authority.
pub struct EvidencePaths<'a> {
    pub output_contract: &'a str,
    pub parser_coverage: &'a str,
    pub report_map: &'a str,
    pub slurm_run_id: &'a str,
}
/// Explicit local root and independently substitutable evidence adapters.
pub struct EvidenceContext<'a> {
    pub repo_root: &'a Path,
    pub serving: &'a dyn ServingEvidence,
    pub output: &'a dyn OutputEvidence,
    pub reporting: &'a dyn ReportEvidence,
    pub consistency: &'a dyn ConsistencyEvidence,
    pub paths: EvidencePaths<'a>,
}
