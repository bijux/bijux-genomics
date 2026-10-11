//! Native version observation through the shared process lifecycle and bounded stream owner.
use super::{
    ExternalArtifactProbeObservation, ExternalArtifactProbeOutcome, ExternalArtifactProbeRequest,
    ExternalArtifactProbeTransport,
};
use anyhow::{bail, Result};
use std::process::Command;

/// Local process transport. Unix capture owns an isolated process group and bounded streams;
/// platforms without that capture implementation reject before launching the command.
#[derive(Default)]
pub struct NativeExternalArtifactProbeTransport;

impl ExternalArtifactProbeTransport for NativeExternalArtifactProbeTransport {
    fn observe(
        &mut self,
        request: &ExternalArtifactProbeRequest<'_>,
    ) -> Result<ExternalArtifactProbeObservation> {
        let [program, arguments @ ..] = request.command else {
            bail!("external artifact version probe is empty");
        };
        let output = bijux_dna_process::capture(
            Command::new(program).args(arguments),
            bijux_dna_process::CaptureLimits {
                timeout: request.timeout,
                max_output_bytes: request.max_output_bytes,
            },
            "external artifact version probe",
        )?;
        let outcome = if output.timed_out {
            ExternalArtifactProbeOutcome::TimedOut
        } else if output.output_limit_exceeded {
            ExternalArtifactProbeOutcome::OutputLimitExceeded
        } else {
            ExternalArtifactProbeOutcome::Exited { code: output.status.code() }
        };
        Ok(ExternalArtifactProbeObservation {
            outcome,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}
