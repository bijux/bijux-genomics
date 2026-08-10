use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;

use anyhow::{anyhow, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const MAX_RETAINED_ATTEMPT_NUMBER: u32 = 999;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApptainerBuildFailureClass {
    TransientTransport,
    Permanent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApptainerBuildAttempt {
    pub attempt: u32,
    pub log_path: PathBuf,
    pub started_at_unix_ms: u64,
    pub finished_at_unix_ms: u64,
    pub failure_class: Option<ApptainerBuildFailureClass>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApptainerBuildReport {
    pub attempts: Vec<ApptainerBuildAttempt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApptainerBuildRequest {
    pub executable: PathBuf,
    pub definition_path: PathBuf,
    pub image_path: PathBuf,
    pub evidence_root: PathBuf,
    pub tool_id: String,
    pub build_evidence_id: String,
}

impl ApptainerBuildRequest {
    #[must_use]
    pub fn retained(
        definition_path: PathBuf,
        image_path: PathBuf,
        containers_root: &Path,
        tool_id: String,
        build_evidence_id: String,
    ) -> Self {
        Self {
            executable: PathBuf::from("apptainer"),
            definition_path,
            image_path,
            evidence_root: containers_root.join("evidence/build-logs"),
            tool_id,
            build_evidence_id,
        }
    }
}

/// Execute one Apptainer image build attempt and retain its append-only evidence log.
///
/// # Errors
/// Returns an error when the request is invalid, an attempt log cannot be created, Apptainer
/// cannot be started, or the final build attempt fails.
pub fn build_apptainer_image(request: &ApptainerBuildRequest) -> Result<ApptainerBuildReport> {
    validate_request(request)?;
    std::fs::create_dir_all(&request.evidence_root).with_context(|| {
        format!("create Apptainer build evidence root {}", request.evidence_root.display())
    })?;
    if let Some(parent) = request.image_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create Apptainer image root {}", parent.display()))?;
    }

    let attempt = next_apptainer_build_attempt(
        &request.evidence_root,
        &request.tool_id,
        &request.build_evidence_id,
    )?;
    let log_path = apptainer_build_log_path(
        &request.evidence_root,
        &request.tool_id,
        &request.build_evidence_id,
        attempt,
    );
    let output = run_build_attempt(request, attempt, &log_path)?;
    if output.success {
        return Ok(ApptainerBuildReport {
            attempts: vec![ApptainerBuildAttempt {
                attempt,
                log_path,
                started_at_unix_ms: output.started_at_unix_ms,
                finished_at_unix_ms: output.finished_at_unix_ms,
                failure_class: None,
            }],
        });
    }

    let failure_class = classify_apptainer_build_failure(&output.diagnostics);
    append_failure_class(&log_path, failure_class)?;
    remove_partial_image(&request.image_path)?;
    Err(anyhow!(
        "Apptainer build failed for {} on attempt {attempt} ({failure_class:?}); inspect {}",
        request.definition_path.display(),
        log_path.display()
    ))
}

#[must_use]
pub fn classify_apptainer_build_failure(diagnostics: &str) -> ApptainerBuildFailureClass {
    let normalized = diagnostics.to_ascii_lowercase();
    let transient_markers = [
        "unexpected end of json input",
        "unexpected eof",
        "connection reset by peer",
        "connection timed out",
        "i/o timeout",
        "tls handshake timeout",
        "temporary failure in name resolution",
        "network is unreachable",
        "toomanyrequests",
        "too many requests",
        "service unavailable",
        "curl: (18)",
        "transfer closed with outstanding read data remaining",
    ];
    if transient_markers.iter().any(|marker| normalized.contains(marker))
        || has_transient_http_status(&normalized)
    {
        ApptainerBuildFailureClass::TransientTransport
    } else {
        ApptainerBuildFailureClass::Permanent
    }
}

fn has_transient_http_status(diagnostics: &str) -> bool {
    ["429", "500", "502", "503", "504"].iter().any(|status| {
        [
            format!("http {status}"),
            format!("http status {status}"),
            format!("status code {status}"),
            format!("returned error: {status}"),
            format!("response status: {status}"),
        ]
        .iter()
        .any(|marker| diagnostics.contains(marker))
    })
}

#[must_use]
pub fn apptainer_build_log_path(
    evidence_root: &Path,
    tool_id: &str,
    build_evidence_id: &str,
    attempt: u32,
) -> PathBuf {
    evidence_root.join(format!("{tool_id}-{build_evidence_id}-attempt-{attempt:03}.log"))
}

#[must_use]
pub fn latest_apptainer_build_attempt(
    evidence_root: &Path,
    tool_id: &str,
    build_evidence_id: &str,
) -> Option<(u32, PathBuf)> {
    (1..=MAX_RETAINED_ATTEMPT_NUMBER)
        .map(|attempt| {
            (attempt, apptainer_build_log_path(evidence_root, tool_id, build_evidence_id, attempt))
        })
        .rfind(|(_, path)| path.is_file())
}

/// Return the first available append-only attempt identity.
///
/// # Errors
/// Returns an error when all governed attempt identities have already been consumed.
pub fn next_apptainer_build_attempt(
    evidence_root: &Path,
    tool_id: &str,
    build_evidence_id: &str,
) -> Result<u32> {
    let next = latest_apptainer_build_attempt(evidence_root, tool_id, build_evidence_id)
        .map_or(1, |(attempt, _)| attempt + 1);
    if next > MAX_RETAINED_ATTEMPT_NUMBER {
        return Err(anyhow!(
            "Apptainer build exceeds {MAX_RETAINED_ATTEMPT_NUMBER} attempts for {tool_id}"
        ));
    }
    Ok(next)
}

struct BuildAttemptOutput {
    success: bool,
    diagnostics: String,
    started_at_unix_ms: u64,
    finished_at_unix_ms: u64,
}

fn run_build_attempt(
    request: &ApptainerBuildRequest,
    attempt: u32,
    log_path: &Path,
) -> Result<BuildAttemptOutput> {
    let mut log = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(log_path)
        .with_context(|| format!("create immutable build log {}", log_path.display()))?;
    let started_at_unix_ms = unix_time_millis()?;
    writeln!(log, "attempt={attempt}")?;
    writeln!(log, "started_at_unix_ms={started_at_unix_ms}")?;
    writeln!(log, "executable={}", request.executable.display())?;
    writeln!(log, "image={}", request.image_path.display())?;
    writeln!(log, "definition={}", request.definition_path.display())?;
    let stdout =
        log.try_clone().with_context(|| format!("clone build log {}", log_path.display()))?;
    let status = Command::new(&request.executable)
        .args(["build", "--force"])
        .arg(&request.image_path)
        .arg(&request.definition_path)
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(log))
        .status()
        .with_context(|| {
            format!("execute Apptainer build for {}", request.definition_path.display())
        })?;
    let finished_at_unix_ms = unix_time_millis()?;
    let mut terminal_log = OpenOptions::new()
        .append(true)
        .open(log_path)
        .with_context(|| format!("append terminal build status to {}", log_path.display()))?;
    writeln!(terminal_log, "finished_at_unix_ms={finished_at_unix_ms}")?;
    writeln!(terminal_log, "exit_status={status}")?;
    let diagnostics = std::fs::read_to_string(log_path)
        .with_context(|| format!("read Apptainer build log {}", log_path.display()))?;
    Ok(BuildAttemptOutput {
        success: status.success(),
        diagnostics,
        started_at_unix_ms,
        finished_at_unix_ms,
    })
}

fn unix_time_millis() -> Result<u64> {
    let millis = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("system clock precedes Unix epoch")?
        .as_millis();
    u64::try_from(millis).context("Unix timestamp exceeds u64")
}

fn validate_request(request: &ApptainerBuildRequest) -> Result<()> {
    if !is_safe_component(&request.tool_id) {
        return Err(anyhow!("Apptainer build tool ID is not a safe path component"));
    }
    if request.build_evidence_id.len() != 64
        || !request
            .build_evidence_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || request.build_evidence_id.bytes().all(|byte| byte == b'0')
    {
        return Err(anyhow!("Apptainer build evidence ID is not a nonzero SHA-256 digest"));
    }
    if !request.definition_path.is_file() {
        return Err(anyhow!(
            "Apptainer definition is missing: {}",
            request.definition_path.display()
        ));
    }
    Ok(())
}

fn append_failure_class(path: &Path, failure_class: ApptainerBuildFailureClass) -> Result<()> {
    let mut log = OpenOptions::new()
        .append(true)
        .open(path)
        .with_context(|| format!("append build failure class to {}", path.display()))?;
    writeln!(log, "failure_class={}", failure_class.as_str())?;
    Ok(())
}

impl ApptainerBuildFailureClass {
    const fn as_str(self) -> &'static str {
        match self {
            Self::TransientTransport => "transient_transport",
            Self::Permanent => "permanent",
        }
    }
}

fn is_safe_component(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn remove_partial_image(path: &Path) -> Result<()> {
    if path.exists() {
        std::fs::remove_file(path)
            .with_context(|| format!("remove partial Apptainer image {}", path.display()))?;
    }
    Ok(())
}
