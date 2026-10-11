use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use regex::Regex;

use super::super::tool_contract::{
    catalog_identity_at_commit, load_container_catalog, repository_head_commit,
};
use super::super::EvidenceSourceIdentity;
use super::model::{
    ExternalArtifactAdmission, ExternalArtifactAdmissionStatus, ExternalArtifactCommandBinding,
    ExternalArtifactIdentity, ExternalArtifactKind, ExternalArtifactVersionProbe,
    EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V1,
};

const MAX_VERSION_OUTPUT_BYTES: usize = 64 * 1024;

pub struct ExternalArtifactInput<'a> {
    pub kind: ExternalArtifactKind,
    pub path: &'a Path,
    pub expected_sha256: &'a str,
    pub lawful_access_basis: &'a str,
    pub apptainer_executable: Option<&'a Path>,
}

pub struct ExternalArtifactAdmissionRequest<'a> {
    pub repository_root: &'a Path,
    pub distributable_image_root: &'a Path,
    pub repository_commit: &'a str,
    pub tool_id: &'a str,
    pub recorded_at_utc: &'a str,
    pub artifact: Option<ExternalArtifactInput<'a>>,
    pub blocker_reason: Option<&'a str>,
}

/// Admit an operator-provided artifact without copying it into repository or active image storage.
///
/// # Errors
/// Returns an error when catalog capability, source identity, access basis, checksum, version, or
/// no-clobber publication requirements are not satisfied.
pub fn admit_external_artifact(
    request: &ExternalArtifactAdmissionRequest<'_>,
    output: &Path,
) -> Result<ExternalArtifactAdmission> {
    require_timestamp(request.recorded_at_utc)?;
    let repository_root = request
        .repository_root
        .canonicalize()
        .context("canonicalize external-artifact repository root")?;
    let distributable_image_root = request
        .distributable_image_root
        .canonicalize()
        .context("canonicalize distributable image root")?;
    let head = repository_head_commit(&repository_root)?;
    if head != request.repository_commit {
        bail!("external-artifact admission commit differs from repository HEAD");
    }
    let catalog_path = repository_root.join("containers/contracts/catalog.toml");
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(&catalog_path)
        .context("hash external-artifact catalog")?;
    if catalog_identity_at_commit(&repository_root, request.repository_commit)? != catalog_sha256 {
        bail!("external-artifact catalog differs from the requested repository commit");
    }
    let catalog = load_container_catalog(&repository_root)?;
    let tool = catalog
        .tools
        .into_iter()
        .find(|tool| tool.id == request.tool_id)
        .with_context(|| format!("container catalog omits tool {}", request.tool_id))?;
    if tool.support_disposition != "unsupported" || !tool.external_artifact_supported {
        bail!("tool {} does not permit external artifact admission", request.tool_id);
    }
    if tool.apptainer_supported || tool.docker_arm64_supported {
        bail!("external artifact tool {} exposes a distributable image", request.tool_id);
    }

    let (status, lawful_access_basis, artifact, version_probe, blocker_reason) =
        match (&request.artifact, request.blocker_reason) {
            (Some(input), None) => {
                let admitted =
                    inspect_artifact(&repository_root, &distributable_image_root, &tool, input)?;
                (
                    ExternalArtifactAdmissionStatus::Admitted,
                    Some(input.lawful_access_basis.trim().to_string()),
                    Some(admitted.0),
                    Some(admitted.1),
                    None,
                )
            }
            (None, Some(reason)) if !reason.trim().is_empty() => (
                ExternalArtifactAdmissionStatus::BlockedExternalAccess,
                None,
                None,
                None,
                Some(reason.trim().to_string()),
            ),
            (Some(_), Some(_)) => {
                bail!("external artifact admission cannot be both admitted and blocked")
            }
            (None, _) => bail!("external artifact admission requires an artifact or blocker"),
        };

    let admission = ExternalArtifactAdmission {
        schema_version: EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V1.to_string(),
        status,
        recorded_at_utc: request.recorded_at_utc.to_string(),
        repository_commit: request.repository_commit.to_string(),
        catalog: EvidenceSourceIdentity {
            path: "containers/contracts/catalog.toml".to_string(),
            sha256: catalog_sha256,
        },
        distributable_image_root: distributable_image_root.display().to_string(),
        tool_id: tool.id,
        version: tool.version,
        support_disposition: tool.support_disposition,
        support_reason: tool.support_reason,
        stage_ids: tool.stage_ids,
        expected_binary: tool.expected_bin,
        version_command: tool.version_command,
        expected_version_regex: tool.expected_version_regex,
        redistribution_permitted: false,
        lawful_access_basis,
        artifact,
        version_probe,
        blocker_reason,
    };
    validate_external_artifact_admission(&admission)?;
    bijux_dna_infra::atomic_create_json(output, &admission)
        .context("publish external-artifact admission")?;
    Ok(admission)
}

/// Load and validate an external-artifact admission record.
///
/// # Errors
/// Returns an error when the document is malformed or violates lifecycle invariants.
pub fn load_external_artifact_admission(path: &Path) -> Result<ExternalArtifactAdmission> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, 4 * 1024 * 1024)
        .context("read external-artifact admission")?;
    let admission: ExternalArtifactAdmission =
        serde_json::from_slice(&bytes).context("parse external-artifact admission")?;
    validate_external_artifact_admission(&admission)?;
    Ok(admission)
}

/// Reverify catalog and artifact identities without executing a stage command.
///
/// # Errors
/// Returns an error when catalog capability changed or an admitted artifact drifted.
pub fn verify_external_artifact_admission(
    repository_root: &Path,
    distributable_image_root: &Path,
    admission_path: &Path,
) -> Result<ExternalArtifactAdmission> {
    let admission = load_external_artifact_admission(admission_path)?;
    let repository_root =
        repository_root.canonicalize().context("canonicalize external-artifact repository root")?;
    let distributable_image_root = distributable_image_root
        .canonicalize()
        .context("canonicalize requested distributable image root")?;
    if Path::new(&admission.distributable_image_root) != distributable_image_root {
        bail!("external-artifact distributable image root changed");
    }
    let catalog_path = repository_root.join(&admission.catalog.path);
    let current_catalog_sha256 =
        bijux_dna_infra::hash_file_sha256(&catalog_path).context("hash current catalog")?;
    if current_catalog_sha256 != admission.catalog.sha256 {
        bail!("external-artifact catalog identity changed");
    }
    if catalog_identity_at_commit(&repository_root, &admission.repository_commit)?
        != admission.catalog.sha256
    {
        bail!("external-artifact repository commit does not contain the admitted catalog");
    }
    let catalog = load_container_catalog(&repository_root)?;
    let tool = catalog
        .tools
        .iter()
        .find(|tool| tool.id == admission.tool_id)
        .context("external-artifact tool no longer exists in catalog")?;
    if !tool.external_artifact_supported
        || tool.support_disposition != admission.support_disposition
        || tool.support_reason != admission.support_reason
        || tool.version != admission.version
        || tool.stage_ids != admission.stage_ids
        || tool.expected_bin != admission.expected_binary
        || tool.version_command != admission.version_command
        || tool.expected_version_regex != admission.expected_version_regex
    {
        bail!("external-artifact tool contract changed");
    }
    if let Some(artifact) = &admission.artifact {
        let path = Path::new(&artifact.path);
        if path.starts_with(&repository_root) {
            bail!("external artifact entered the product repository");
        }
        if path.starts_with(&distributable_image_root) {
            bail!("external artifact entered the distributable active image root");
        }
        if !tool.external_artifact_kinds.iter().any(|kind| kind == artifact.kind.as_str()) {
            bail!("external artifact kind is no longer permitted");
        }
        let metadata = path.metadata().context("inspect external artifact")?;
        if !metadata.is_file()
            || metadata.len() != artifact.size_bytes
            || bijux_dna_infra::hash_file_sha256(path).context("rehash external artifact")?
                != artifact.sha256
        {
            bail!("external artifact identity changed");
        }
        validate_persisted_version_probe(&admission, artifact)?;
    }
    Ok(admission)
}

/// Bind a stage command to a reverified operator artifact.
///
/// # Errors
/// Returns an error before execution when access is blocked, identity drifted, the stage is not
/// allowed, or the command does not invoke the cataloged binary.
pub fn bind_external_artifact_command(
    repository_root: &Path,
    distributable_image_root: &Path,
    admission_path: &Path,
    stage_id: &str,
    command: &[String],
    apptainer_executable: Option<&Path>,
) -> Result<ExternalArtifactCommandBinding> {
    let admission = verify_external_artifact_admission(
        repository_root,
        distributable_image_root,
        admission_path,
    )?;
    if admission.status == ExternalArtifactAdmissionStatus::BlockedExternalAccess {
        bail!(
            "external artifact access blocked for {}: {}",
            admission.tool_id,
            admission.blocker_reason.as_deref().unwrap_or("unspecified")
        );
    }
    if !admission.stage_ids.iter().any(|allowed| allowed == stage_id) {
        bail!("tool {} is not bound to stage {stage_id}", admission.tool_id);
    }
    let [program, arguments @ ..] = command else {
        bail!("external artifact stage command is empty");
    };
    let artifact = admission.artifact.context("admitted external artifact has no identity")?;
    let (executable, arguments) = match artifact.kind {
        ExternalArtifactKind::Binary => {
            bind_binary_command(program, arguments, &admission.expected_binary, &artifact.path)?
        }
        ExternalArtifactKind::ApptainerSif => {
            require_expected_binary_invocation(command, &admission.expected_binary)?;
            let executable =
                apptainer_executable.map_or_else(|| PathBuf::from("apptainer"), Path::to_path_buf);
            let mut bound = vec!["exec".to_string(), artifact.path];
            bound.extend(command.iter().cloned());
            (executable.display().to_string(), bound)
        }
    };
    Ok(ExternalArtifactCommandBinding {
        tool_id: admission.tool_id,
        stage_id: stage_id.to_string(),
        artifact_sha256: artifact.sha256,
        executable,
        arguments,
    })
}

fn bind_binary_command(
    program: &str,
    arguments: &[String],
    expected_binary: &str,
    artifact_path: &str,
) -> Result<(String, Vec<String>)> {
    if program == expected_binary {
        return Ok((artifact_path.to_string(), arguments.to_vec()));
    }
    let [shell_option, script, remaining @ ..] = arguments else {
        bail!("external artifact stage command must invoke {expected_binary}");
    };
    if !matches!(program, "sh" | "/bin/sh" | "bash" | "/bin/bash")
        || !matches!(shell_option.as_str(), "-c" | "-lc")
    {
        bail!("external artifact stage command must invoke {expected_binary}");
    }
    let bound_script = replace_shell_command_word(script, expected_binary, artifact_path)?;
    let mut bound_arguments = vec![shell_option.clone(), bound_script];
    bound_arguments.extend_from_slice(remaining);
    Ok((program.to_string(), bound_arguments))
}

fn require_expected_binary_invocation(command: &[String], expected_binary: &str) -> Result<()> {
    let [program, arguments @ ..] = command else {
        bail!("external artifact stage command is empty");
    };
    if program == expected_binary {
        return Ok(());
    }
    let [shell_option, script, ..] = arguments else {
        bail!("external artifact stage command must invoke {expected_binary}");
    };
    if !matches!(program.as_str(), "sh" | "/bin/sh" | "bash" | "/bin/bash")
        || !matches!(shell_option.as_str(), "-c" | "-lc")
    {
        bail!("external artifact stage command must invoke {expected_binary}");
    }
    replace_shell_command_word(script, expected_binary, expected_binary).map(|_| ())
}

fn replace_shell_command_word(
    script: &str,
    expected_binary: &str,
    replacement: &str,
) -> Result<String> {
    let pattern = Regex::new(&format!(
        r"(?m)(^|[;&|])([[:space:]]*){}([[:space:]]|$)",
        regex::escape(expected_binary)
    ))
    .context("compile external artifact shell binding")?;
    if !pattern.is_match(script) {
        bail!("external artifact shell command must invoke {expected_binary}");
    }
    let quoted = shell_quote(replacement);
    Ok(pattern
        .replace_all(script, |captures: &regex::Captures<'_>| {
            format!("{}{}{}{}", &captures[1], &captures[2], quoted, &captures[3])
        })
        .into_owned())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn inspect_artifact(
    repository_root: &Path,
    distributable_image_root: &Path,
    tool: &crate::evidence::tool_contract::ContainerCatalogTool,
    input: &ExternalArtifactInput<'_>,
) -> Result<(ExternalArtifactIdentity, ExternalArtifactVersionProbe)> {
    if input.lawful_access_basis.trim().is_empty() {
        bail!("external artifact requires a lawful access basis");
    }
    if !tool.external_artifact_kinds.iter().any(|kind| kind == input.kind.as_str()) {
        bail!("tool {} does not permit {} artifacts", tool.id, input.kind.as_str());
    }
    validate_sha256(input.expected_sha256, "external artifact checksum")?;
    let artifact_path = input.path.canonicalize().context("canonicalize external artifact")?;
    if artifact_path.starts_with(repository_root) {
        bail!("external artifact must remain outside the product repository");
    }
    if artifact_path.starts_with(distributable_image_root) {
        bail!("external artifact must remain outside the distributable active image root");
    }
    let metadata = artifact_path.metadata().context("inspect external artifact")?;
    if !metadata.is_file() {
        bail!("external artifact is not a regular file");
    }
    let sha256 =
        bijux_dna_infra::hash_file_sha256(&artifact_path).context("hash external artifact")?;
    if sha256 != input.expected_sha256.to_ascii_lowercase() {
        bail!("external artifact checksum does not match operator declaration");
    }
    let command = version_probe_command(tool, input, &artifact_path)?;
    let output = execute_version_probe(&command)?;
    let expected = Regex::new(&tool.expected_version_regex)
        .context("compile external artifact version contract")?;
    if !expected.is_match(&output) {
        bail!("external artifact version output does not match catalog contract");
    }
    Ok((
        ExternalArtifactIdentity {
            kind: input.kind,
            path: artifact_path.display().to_string(),
            sha256,
            size_bytes: metadata.len(),
        },
        ExternalArtifactVersionProbe {
            command,
            output_sha256: bijux_dna_infra::hash_bytes_sha256(output.as_bytes()),
            output,
        },
    ))
}

fn version_probe_command(
    tool: &crate::evidence::tool_contract::ContainerCatalogTool,
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
    let mut command = match input.kind {
        ExternalArtifactKind::Binary => vec![artifact_path.display().to_string()],
        ExternalArtifactKind::ApptainerSif => vec![
            input
                .apptainer_executable
                .map_or_else(|| PathBuf::from("apptainer"), Path::to_path_buf)
                .display()
                .to_string(),
            "exec".to_string(),
            artifact_path.display().to_string(),
            tool.expected_bin.clone(),
        ],
    };
    command.extend(arguments.iter().map(|argument| (*argument).to_string()));
    Ok(command)
}

fn execute_version_probe(command: &[String]) -> Result<String> {
    let [program, arguments @ ..] = command else {
        bail!("external artifact version probe is empty");
    };
    let output = Command::new(program)
        .args(arguments)
        .output()
        .with_context(|| format!("execute external artifact version probe {program}"))?;
    let byte_count = output.stdout.len().saturating_add(output.stderr.len());
    if byte_count > MAX_VERSION_OUTPUT_BYTES {
        bail!("external artifact version output exceeds the bounded evidence limit");
    }
    if !output.status.success() {
        bail!("external artifact version probe exited with {}", output.status);
    }
    let mut bytes = output.stdout;
    if !bytes.is_empty() && !output.stderr.is_empty() && !bytes.ends_with(b"\n") {
        bytes.push(b'\n');
    }
    bytes.extend(output.stderr);
    String::from_utf8(bytes).context("external artifact version output is not UTF-8")
}

fn validate_persisted_version_probe(
    admission: &ExternalArtifactAdmission,
    artifact: &ExternalArtifactIdentity,
) -> Result<()> {
    let probe = admission.version_probe.as_ref().context("admission omits version probe")?;
    let tokens = admission.version_command.split_ascii_whitespace().collect::<Vec<_>>();
    let [program, version_arguments @ ..] = tokens.as_slice() else {
        bail!("external artifact version command is empty");
    };
    if *program != admission.expected_binary {
        bail!("external artifact version command does not invoke the expected binary");
    }
    let valid_command = match artifact.kind {
        ExternalArtifactKind::Binary => {
            probe.command.first().is_some_and(|value| value == &artifact.path)
                && probe.command[1..]
                    == version_arguments
                        .iter()
                        .map(|argument| (*argument).to_string())
                        .collect::<Vec<_>>()
        }
        ExternalArtifactKind::ApptainerSif => {
            probe.command.len() >= 4
                && probe.command[1] == "exec"
                && probe.command[2] == artifact.path
                && probe.command[3] == admission.expected_binary
                && probe.command[4..]
                    == version_arguments
                        .iter()
                        .map(|argument| (*argument).to_string())
                        .collect::<Vec<_>>()
        }
    };
    let expected = Regex::new(&admission.expected_version_regex)
        .context("compile persisted external artifact version contract")?;
    if !valid_command || !expected.is_match(&probe.output) {
        bail!("external artifact version probe evidence changed");
    }
    Ok(())
}

fn validate_external_artifact_admission(admission: &ExternalArtifactAdmission) -> Result<()> {
    if admission.schema_version != EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V1 {
        bail!("unsupported external-artifact admission schema");
    }
    require_timestamp(&admission.recorded_at_utc)?;
    validate_sha256(&admission.catalog.sha256, "external artifact catalog")?;
    if admission.catalog.path != "containers/contracts/catalog.toml" {
        bail!("external-artifact catalog path is not canonical");
    }
    if admission.repository_commit.len() != 40
        || !admission.repository_commit.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("external-artifact repository commit is invalid");
    }
    if admission.tool_id.trim().is_empty()
        || admission.version.trim().is_empty()
        || admission.support_disposition != "unsupported"
        || admission.support_reason.trim().is_empty()
        || admission.distributable_image_root.trim().is_empty()
        || !Path::new(&admission.distributable_image_root).is_absolute()
        || admission.stage_ids.is_empty()
        || admission.expected_binary.trim().is_empty()
        || admission.version_command.trim().is_empty()
        || admission.expected_version_regex.trim().is_empty()
        || admission.redistribution_permitted
    {
        bail!("external-artifact admission has an incomplete or distributable tool contract");
    }
    let mut stages = admission.stage_ids.clone();
    stages.sort();
    stages.dedup();
    if stages != admission.stage_ids {
        bail!("external-artifact stage bindings must be sorted and unique");
    }
    match admission.status {
        ExternalArtifactAdmissionStatus::Admitted => {
            let basis = admission.lawful_access_basis.as_deref().unwrap_or_default();
            let artifact = admission.artifact.as_ref().context("admission omits artifact")?;
            let probe =
                admission.version_probe.as_ref().context("admission omits version probe")?;
            if basis.trim().is_empty() || admission.blocker_reason.is_some() {
                bail!("admitted external artifact has invalid access state");
            }
            validate_sha256(&artifact.sha256, "external artifact")?;
            validate_sha256(&probe.output_sha256, "external artifact version output")?;
            if artifact.path.trim().is_empty()
                || artifact.size_bytes == 0
                || probe.command.is_empty()
                || probe.output.trim().is_empty()
                || bijux_dna_infra::hash_bytes_sha256(probe.output.as_bytes())
                    != probe.output_sha256
            {
                bail!("admitted external artifact has incomplete identity evidence");
            }
        }
        ExternalArtifactAdmissionStatus::BlockedExternalAccess => {
            if admission.artifact.is_some()
                || admission.version_probe.is_some()
                || admission.lawful_access_basis.is_some()
                || admission.blocker_reason.as_deref().unwrap_or_default().trim().is_empty()
            {
                bail!("blocked external artifact has invalid access state");
            }
        }
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("{label} SHA-256 identity is invalid");
    }
    Ok(())
}

fn require_timestamp(value: &str) -> Result<()> {
    if value.trim().is_empty() || !value.ends_with('Z') || !value.contains('T') {
        bail!("external-artifact timestamp is not UTC RFC 3339");
    }
    Ok(())
}
