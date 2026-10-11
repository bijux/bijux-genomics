//! Owner: canonical encrypted campaign submission payload encoding.
use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    assemble_hpc_code_submission, assemble_hpc_results_submission, encode_redacted_hpc_bundle,
    HpcBundleFoundationLocks, HpcBundleSubmission, HpcCodeFreezeObservation,
};

pub(crate) fn encode_results_submission(
    input: &HpcBundleSubmission<'_>,
    needles: &[String],
) -> Result<Vec<u8>> {
    let value = serde_json::to_value(assemble_hpc_results_submission(input))
        .context("serialize results bundle payload")?;
    encode_redacted_hpc_bundle(value, needles).context("serialize results bundle payload")
}

pub(crate) fn encode_code_submission(
    input: &HpcBundleSubmission<'_>,
    observation: HpcCodeFreezeObservation,
    locks: HpcBundleFoundationLocks,
    needles: &[String],
) -> Result<Vec<u8>> {
    let value = serde_json::to_value(assemble_hpc_code_submission(input, observation, locks))
        .context("serialize code bundle payload")?;
    encode_redacted_hpc_bundle(value, needles).context("serialize code bundle payload")
}
