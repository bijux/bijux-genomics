//! Deterministic stage command projection from a reverified external-artifact admission.
use super::model::{
    ExternalArtifactAdmission, ExternalArtifactAdmissionStatus, ExternalArtifactCommandBinding,
    ExternalArtifactKind,
};
use anyhow::{bail, Context, Result};
use regex::Regex;
use std::path::{Path, PathBuf};

pub(super) fn bind_verified_external_artifact_command(
    admission: ExternalArtifactAdmission,
    stage_id: &str,
    command: &[String],
    apptainer_executable: Option<&Path>,
) -> Result<ExternalArtifactCommandBinding> {
    match admission.availability {
        ExternalArtifactAdmissionStatus::Admitted => {}
        ExternalArtifactAdmissionStatus::BlockedExternalAccess => {
            bail!(
                "external artifact access blocked for {}: {}",
                admission.tool_id,
                admission.blocker_reason.as_deref().unwrap_or("unspecified")
            );
        }
        ExternalArtifactAdmissionStatus::Unavailable => {
            bail!(
                "external artifact unavailable for {}: {}",
                admission.tool_id,
                admission.blocker_reason.as_deref().unwrap_or("unspecified")
            );
        }
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
