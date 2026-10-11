//! Version command projection, bounded observation contracts and evidence admission.
mod native;
pub use native::NativeExternalArtifactProbeTransport;

use super::model::{ExternalArtifactKind, ExternalArtifactVersionProbe};
use super::request::ExternalArtifactInput;
use crate::evidence::tool_contract::ContainerCatalogTool;
use anyhow::{bail, Context, Result};
use regex::Regex;
use std::path::Path;
use std::time::Duration;

const MAX_OUTPUT_BYTES: usize = 64 * 1024;
const TIMEOUT: Duration = Duration::from_secs(30);

/// A projected version command and the limits required of its observation transport.
pub struct ExternalArtifactProbeRequest<'a> {
    pub command: &'a [String],
    pub timeout: Duration,
    /// Combined raw stdout and stderr budget; presentation may insert one newline.
    pub max_output_bytes: usize,
}

/// Terminal observation state. Cancellation cannot be mistaken for a successful exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalArtifactProbeOutcome {
    /// `None` denotes termination without an exit code, for example by a signal.
    Exited {
        code: Option<i32>,
    },
    TimedOut,
    OutputLimitExceeded,
}

/// Raw evidence, before exit, byte-count, UTF-8 and catalog-version admission.
pub struct ExternalArtifactProbeObservation {
    pub outcome: ExternalArtifactProbeOutcome,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// Observe the requested argv without interpreting its scientific or version compatibility.
/// Implementations must enforce the deadline and combined byte budget while reading, release
/// owned execution resources on failure, and report unavailable execution as an error.
/// The caller supplies a trusted transport; returned bytes are still admitted independently.
pub trait ExternalArtifactProbeTransport {
    /// # Errors
    /// Returns an error when the requested command cannot be observed within this transport.
    fn observe(
        &mut self,
        request: &ExternalArtifactProbeRequest<'_>,
    ) -> Result<ExternalArtifactProbeObservation>;
}

pub(super) fn probe_version(
    tool: &ContainerCatalogTool,
    input: &ExternalArtifactInput<'_>,
    artifact_path: &Path,
    transport: &mut dyn ExternalArtifactProbeTransport,
) -> Result<ExternalArtifactVersionProbe> {
    let command = project_command(tool, input, artifact_path)?;
    let expected = Regex::new(&tool.expected_version_regex)
        .context("compile external artifact version contract")?;
    let observed = transport.observe(&ExternalArtifactProbeRequest {
        command: &command,
        timeout: TIMEOUT,
        max_output_bytes: MAX_OUTPUT_BYTES,
    })?;
    let output = admit_output(observed)?;
    if !expected.is_match(&output) {
        bail!("external artifact version output does not match catalog contract");
    }
    Ok(ExternalArtifactVersionProbe {
        command,
        output_sha256: bijux_dna_infra::hash_bytes_sha256(output.as_bytes()),
        output,
    })
}

fn project_command(
    tool: &ContainerCatalogTool,
    input: &ExternalArtifactInput<'_>,
    artifact_path: &Path,
) -> Result<Vec<String>> {
    let tokens = tool.version_command.split_ascii_whitespace().collect::<Vec<_>>();
    let [program, arguments @ ..] = tokens.as_slice() else {
        bail!("external artifact version command is empty");
    };
    if *program != tool.expected_bin {
        bail!("external artifact version command does not invoke the expected binary");
    }
    let artifact = artifact_path.to_str().context("external artifact path is not UTF-8")?;
    let mut command = match input.kind {
        ExternalArtifactKind::Binary => vec![artifact.to_string()],
        ExternalArtifactKind::ApptainerSif => vec![
            input
                .apptainer_executable
                .unwrap_or_else(|| Path::new("apptainer"))
                .to_str()
                .context("external artifact runtime path is not UTF-8")?
                .to_string(),
            "exec".to_string(),
            artifact.to_string(),
            tool.expected_bin.clone(),
        ],
    };
    command.extend(arguments.iter().map(|argument| (*argument).to_string()));
    if command[0].is_empty() || command.iter().any(|argument| argument.contains('\0')) {
        bail!("external artifact version command contains an invalid argument");
    }
    Ok(command)
}

fn admit_output(observed: ExternalArtifactProbeObservation) -> Result<String> {
    match observed.outcome {
        ExternalArtifactProbeOutcome::TimedOut => {
            bail!("external artifact version probe exceeded its 30-second deadline")
        }
        ExternalArtifactProbeOutcome::OutputLimitExceeded => {
            bail!("external artifact version output exceeds the bounded evidence limit")
        }
        ExternalArtifactProbeOutcome::Exited { code: Some(0) } => {}
        ExternalArtifactProbeOutcome::Exited { code } => {
            bail!("external artifact version probe exited unsuccessfully with code {code:?}")
        }
    }
    if observed.stdout.len().saturating_add(observed.stderr.len()) > MAX_OUTPUT_BYTES {
        bail!("external artifact version output exceeds the bounded evidence limit");
    }
    let mut bytes = observed.stdout;
    if !bytes.is_empty() && !observed.stderr.is_empty() && !bytes.ends_with(b"\n") {
        bytes.push(b'\n');
    }
    bytes.extend(observed.stderr);
    String::from_utf8(bytes).context("external artifact version output is not UTF-8")
}
