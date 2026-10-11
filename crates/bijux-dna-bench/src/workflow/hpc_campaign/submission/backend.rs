//! Owner: scheduler and encrypted publication ports for campaign submission.
use anyhow::Result;
use bijux_dna_bench_model::{HpcBundleSubmission, HpcCodeFreezeObservation, PlannedJob};
use std::path::Path;

/// External execution capabilities supplied by the application composition boundary.
pub trait HpcSubmissionBackend {
    /// Submit one prepared job and return its admitted scheduler receipt identity.
    /// # Errors
    /// Propagates transport failures and malformed scheduler responses.
    fn submit_job(
        &self,
        job: &PlannedJob,
        script: &Path,
        dependencies: &[String],
        submitted_at: &str,
    ) -> Result<String>;
    /// Observe current code and script identities for the submission bundle.
    /// # Errors
    /// Propagates required source observation failures.
    fn observe_code_freeze(&self, script: &Path) -> Result<HpcCodeFreezeObservation>;
    /// Publish one encrypted payload and its transport sidecar.
    /// # Errors
    /// Propagates encryption or publication failures without inventing a successful receipt.
    fn publish_bundle(
        &self,
        input: &HpcBundleSubmission<'_>,
        output: &Path,
        kind: &str,
        plaintext: &[u8],
    ) -> Result<()>;
}
