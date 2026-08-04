use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const APPTAINER_SMOKE_MANIFEST_SCHEMA_V5: &str = "bijux.apptainer.smoke_manifest.v5";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApptainerSmokeRequest {
    pub executable: PathBuf,
    pub image_path: PathBuf,
    pub data_root: PathBuf,
    pub results_root: PathBuf,
    pub tool_id: String,
    pub stage_id: String,
    pub build_evidence_id: String,
    pub repository_commit: String,
    pub registry_digest: String,
    pub image_sha256: String,
    pub version_command: String,
    pub expected_version_regex: String,
    pub help_command: String,
    pub require_help: bool,
    pub probe_commands: Vec<String>,
    pub java_heap_mb: Option<u64>,
    pub upstream: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApptainerSmokeProbeResult {
    pub command: String,
    pub applied_command: String,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_first_line: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApptainerSmokeManifest {
    pub schema_version: String,
    pub tool_id: String,
    pub stage_id: String,
    pub status: String,
    pub build_evidence_id: String,
    pub repository_commit: String,
    pub registry_digest: String,
    pub sif_sha256: String,
    pub version_cmd: String,
    pub expected_version_regex: String,
    pub version_matches_contract: bool,
    pub help_cmd: String,
    pub version: String,
    pub version_output_first_line: String,
    pub help_ok: bool,
    pub quick_smoke: bool,
    pub probe_commands: Vec<String>,
    #[serde(default)]
    pub probe_results: Vec<ApptainerSmokeProbeResult>,
    pub java_heap_mb: Option<u64>,
    pub upstream: String,
    pub image_build_timestamp_unix_s: u64,
    pub checked_at_unix_s: u64,
}

/// Execute an offline Apptainer smoke and admit version output against the tool contract regex.
///
/// # Errors
/// Returns an error when request identities, paths, or the expected version regex are invalid.
pub fn run_apptainer_smoke(request: &ApptainerSmokeRequest) -> Result<ApptainerSmokeManifest> {
    validate_request(request)?;
    let version_regex = Regex::new(&request.expected_version_regex)
        .with_context(|| "compile expected tool version regex")?;
    let commands = effective_probe_commands(request);
    let mut probe_results = Vec::with_capacity(commands.len());
    let mut version_output = String::new();
    for (index, command) in commands.iter().enumerate() {
        let applied_command = apply_heap_policy(&request.tool_id, command, request.java_heap_mb);
        match run_apptainer_probe(request, &applied_command) {
            Ok(output) => {
                if index == 0 {
                    version_output.clone_from(&output);
                }
                probe_results.push(ApptainerSmokeProbeResult {
                    command: command.clone(),
                    applied_command,
                    ok: true,
                    output_sha256: Some(sha256_hex(output.as_bytes())),
                    output_first_line: first_line(&output),
                    error: None,
                });
            }
            Err(error) => probe_results.push(ApptainerSmokeProbeResult {
                command: command.clone(),
                applied_command,
                ok: false,
                output_sha256: None,
                output_first_line: None,
                error: Some(format!("{error:#}")),
            }),
        }
    }

    let all_probes_passed = probe_results.iter().all(|result| result.ok);
    let version_match = version_regex.find(&version_output);
    let version_matches_contract = version_match.is_some();
    let help_ok = !request.require_help
        || probe_results
            .iter()
            .find(|result| result.command == request.help_command)
            .is_some_and(|result| result.ok);
    let version_output_first_line = first_line(&version_output).unwrap_or_default();
    let image_build_timestamp_unix_s = std::fs::metadata(&request.image_path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|timestamp| timestamp.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map_or(0, |duration| duration.as_secs());
    Ok(ApptainerSmokeManifest {
        schema_version: APPTAINER_SMOKE_MANIFEST_SCHEMA_V5.to_string(),
        tool_id: request.tool_id.clone(),
        stage_id: request.stage_id.clone(),
        status: if all_probes_passed && help_ok && version_matches_contract {
            "ok".to_string()
        } else {
            "wrapper_failed".to_string()
        },
        build_evidence_id: request.build_evidence_id.clone(),
        repository_commit: request.repository_commit.clone(),
        registry_digest: request.registry_digest.clone(),
        sif_sha256: request.image_sha256.clone(),
        version_cmd: request.version_command.clone(),
        expected_version_regex: request.expected_version_regex.clone(),
        version_matches_contract,
        help_cmd: request.help_command.clone(),
        version: version_match.map_or_else(String::new, |matched| matched.as_str().to_string()),
        version_output_first_line,
        help_ok,
        quick_smoke: true,
        probe_commands: commands,
        probe_results,
        java_heap_mb: request.java_heap_mb,
        upstream: request.upstream.clone(),
        image_build_timestamp_unix_s,
        checked_at_unix_s: now_unix_s(),
    })
}

fn effective_probe_commands(request: &ApptainerSmokeRequest) -> Vec<String> {
    let mut commands = vec![request.version_command.clone()];
    if request.require_help && request.help_command != request.version_command {
        commands.push(request.help_command.clone());
    }
    for probe in &request.probe_commands {
        if !commands.contains(probe) {
            commands.push(probe.clone());
        }
    }
    commands
}

fn run_apptainer_probe(request: &ApptainerSmokeRequest, command: &str) -> Result<String> {
    let mut process = Command::new(&request.executable);
    process.args(["exec", "--containall", "--cleanenv", "--net", "--network", "none"]);
    process.args(["--bind", &format!("{}:/bijux/input:ro", request.data_root.display())]);
    process.args(["--bind", &format!("{}:/bijux/output:rw", request.results_root.display())]);
    let banks_root = request.data_root.join("banks");
    if banks_root.exists() {
        process.args(["--bind", &format!("{}:/bijux/db:ro", banks_root.display())]);
    }
    let output = process
        .arg(&request.image_path)
        .args(["sh", "-lc", command])
        .output()
        .with_context(|| format!("execute Apptainer probe for {}", request.tool_id))?;
    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    if !output.status.success() {
        return Err(anyhow!(
            "Apptainer probe `{command}` failed with {}: {}",
            output.status,
            combined.trim()
        ));
    }
    Ok(combined)
}

fn validate_request(request: &ApptainerSmokeRequest) -> Result<()> {
    for (value, field) in [
        (&request.tool_id, "tool_id"),
        (&request.stage_id, "stage_id"),
        (&request.version_command, "version_command"),
        (&request.expected_version_regex, "expected_version_regex"),
    ] {
        if value.trim().is_empty() || value != value.trim() {
            return Err(anyhow!("Apptainer smoke {field} is empty or malformed"));
        }
    }
    if request.require_help
        && (request.help_command.trim().is_empty()
            || request.help_command != request.help_command.trim())
    {
        return Err(anyhow!("Apptainer smoke help_command is empty or malformed"));
    }
    if request
        .probe_commands
        .iter()
        .any(|command| command.trim().is_empty() || command != command.trim())
    {
        return Err(anyhow!("Apptainer smoke probe command is empty or malformed"));
    }
    for (value, length, field) in [
        (&request.build_evidence_id, 64, "build_evidence_id"),
        (&request.repository_commit, 40, "repository_commit"),
        (&request.registry_digest, 64, "registry_digest"),
        (&request.image_sha256, 64, "image_sha256"),
    ] {
        if value.len() != length
            || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || value.bytes().all(|byte| byte == b'0')
        {
            return Err(anyhow!("Apptainer smoke {field} is not a valid identity"));
        }
    }
    require_existing(&request.image_path, "image")?;
    require_existing(&request.data_root, "input bind root")?;
    require_existing(&request.results_root, "output bind root")?;
    Ok(())
}

fn require_existing(path: &Path, label: &str) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!("Apptainer smoke {label} is missing: {}", path.display()));
    }
    Ok(())
}

fn apply_heap_policy(tool_id: &str, command: &str, java_heap_mb: Option<u64>) -> String {
    let Some(heap_mb) = java_heap_mb else {
        return command.to_string();
    };
    if !matches!(tool_id, "bbduk" | "bbmerge") || command.contains("-Xmx") {
        return command.to_string();
    }
    let mut parts = command.split_whitespace();
    let Some(binary) = parts.next() else {
        return command.to_string();
    };
    let arguments = parts.collect::<Vec<_>>().join(" ");
    if arguments.is_empty() {
        format!("{binary} -Xmx{heap_mb}m")
    } else {
        format!("{binary} -Xmx{heap_mb}m {arguments}")
    }
}

fn first_line(output: &str) -> Option<String> {
    output.lines().next().map(str::trim).filter(|line| !line.is_empty()).map(ToOwned::to_owned)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn now_unix_s() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}
