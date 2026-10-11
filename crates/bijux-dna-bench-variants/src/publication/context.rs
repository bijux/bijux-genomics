//! Observed execution timing and publication addresses for report construction.
use super::PublishedArtifactPaths;
pub(crate) struct ReportContext<'a> {
    pub(crate) paths: PublishedArtifactPaths<'a>,
    pub(crate) started_at: &'a str,
    pub(crate) finished_at: &'a str,
    pub(crate) elapsed_seconds: f64,
}
