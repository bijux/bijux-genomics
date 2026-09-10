//! Owner: typed campaign bundle assembly and ordered encrypted publication.
use super::HpcSubmissionBackend;
use crate::artifacts::hpc_campaign::{encode_code_submission, encode_results_submission};
use crate::repo::hpc_campaign::submission::read_operator_output;
use anyhow::Result;
use bijux_dna_bench_model::{redact_hpc_text, HpcBundleFoundationLocks, HpcBundleSubmission};
use std::path::Path;

pub(super) struct SubmissionBundlePublisher<'a, B> {
    pub locks: &'a HpcBundleFoundationLocks,
    pub backend: &'a B,
    pub redaction_needles: &'a [String],
}

impl<B: HpcSubmissionBackend> SubmissionBundlePublisher<'_, B> {
    pub fn publish_primary(&self, input: &HpcBundleSubmission<'_>) -> Result<()> {
        let results = encode_results_submission(input, self.redaction_needles)?;
        self.backend.publish_bundle(
            input,
            Path::new(&input.job.outputs.results),
            "results",
            &results,
        )?;

        let observation = self.backend.observe_code_freeze(Path::new(input.script_path))?;
        let code =
            encode_code_submission(input, observation, self.locks.clone(), self.redaction_needles)?;
        self.backend.publish_bundle(input, Path::new(&input.job.outputs.code), "code", &code)?;
        Ok(())
    }

    pub fn encrypt_operator_outputs(&self, input: &HpcBundleSubmission<'_>) -> Result<()> {
        if !input.campaign.security.encrypt_operator_outputs {
            return Ok(());
        }
        for (path, kind) in [
            (input.job.outputs.log.as_str(), "operator_log"),
            (input.job.outputs.out.as_str(), "operator_out"),
            (input.job.outputs.err.as_str(), "operator_err"),
        ] {
            let plaintext = read_operator_output(Path::new(path))?;
            let redacted = redact_hpc_text(plaintext, self.redaction_needles);
            self.backend.publish_bundle(input, Path::new(path), kind, redacted.as_bytes())?;
        }
        Ok(())
    }
}
