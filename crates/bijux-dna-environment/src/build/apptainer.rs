use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const MAX_RETAINED_ATTEMPT_NUMBER: u32 = 999;
const MAX_BUILD_INVOCATIONS: u32 = 3;

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
    pub maximum_invocations: u32,
    pub retry_delay: Duration,
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
            maximum_invocations: 3,
            retry_delay: Duration::from_secs(2),
        }
    }
}

/// Build an Apptainer image with append-only attempt logs and bounded transport retries.
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

    let first_attempt = next_apptainer_build_attempt(
        &request.evidence_root,
        &request.tool_id,
        &request.build_evidence_id,
    )?;
    let mut attempts = Vec::new();
    for invocation in 0..request.maximum_invocations {
        let attempt = first_attempt
            .checked_add(invocation)
            .ok_or_else(|| anyhow!("Apptainer build attempt number overflow"))?;
        if attempt > MAX_RETAINED_ATTEMPT_NUMBER {
            return Err(anyhow!(
                "Apptainer build exceeds {MAX_RETAINED_ATTEMPT_NUMBER} attempts for {}",
                request.tool_id
            ));
        }
        let log_path = apptainer_build_log_path(
            &request.evidence_root,
            &request.tool_id,
            &request.build_evidence_id,
            attempt,
        );
        let output = run_build_attempt(request, attempt, &log_path)?;
        if output.success {
            attempts.push(ApptainerBuildAttempt { attempt, log_path, failure_class: None });
            return Ok(ApptainerBuildReport { attempts });
        }

        let failure_class = classify_apptainer_build_failure(&output.diagnostics);
        attempts.push(ApptainerBuildAttempt {
            attempt,
            log_path: log_path.clone(),
            failure_class: Some(failure_class),
        });
        let may_retry = failure_class == ApptainerBuildFailureClass::TransientTransport
            && invocation + 1 < request.maximum_invocations;
        remove_partial_image(&request.image_path)?;
        if !may_retry {
            return Err(anyhow!(
                "Apptainer build failed for {} on attempt {attempt}; inspect {}",
                request.definition_path.display(),
                log_path.display()
            ));
        }
        thread::sleep(request.retry_delay);
    }
    Err(anyhow!("Apptainer build exhausted without a terminal result"))
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
    ];
    if transient_markers.iter().any(|marker| normalized.contains(marker)) {
        ApptainerBuildFailureClass::TransientTransport
    } else {
        ApptainerBuildFailureClass::Permanent
    }
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
    writeln!(log, "attempt={attempt}")?;
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
    let diagnostics = std::fs::read_to_string(log_path)
        .with_context(|| format!("read Apptainer build log {}", log_path.display()))?;
    Ok(BuildAttemptOutput { success: status.success(), diagnostics })
}

fn validate_request(request: &ApptainerBuildRequest) -> Result<()> {
    if !(1..=MAX_BUILD_INVOCATIONS).contains(&request.maximum_invocations) {
        return Err(anyhow!(
            "Apptainer build invocations must be between 1 and {MAX_BUILD_INVOCATIONS}"
        ));
    }
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
