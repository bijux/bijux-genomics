use super::{path_from_arg, read_utf8, success_line, ContainerCommandOutcome, Result, Workspace};

const SCHEMA_PATH: &str = "containers/contracts/schemas/external-artifact-admission.v2.json";
const USAGE: &str = "Usage:\n  bijux-dna-dev containers run external-artifact -- admit <repository-root> <active-image-root> <repository-commit> <tool-id> <binary|apptainer_sif> <artifact> <sha256> <lawful-access-basis> <recorded-at-utc> <output> [<apptainer>]\n  bijux-dna-dev containers run external-artifact -- block <repository-root> <active-image-root> <repository-commit> <tool-id> <recorded-at-utc> <output> <reason>\n  bijux-dna-dev containers run external-artifact -- unavailable <repository-root> <active-image-root> <repository-commit> <tool-id> <recorded-at-utc> <output> <reason>\n  bijux-dna-dev containers run external-artifact -- verify <repository-root> <active-image-root> <record>\n  bijux-dna-dev containers run external-artifact -- bind <repository-root> <active-image-root> <record> <stage-id> <apptainer-or-dash> <program> [<argument> ...]\n  bijux-dna-dev containers run external-artifact -- generate-schema [<output>]\n  bijux-dna-dev containers run external-artifact -- check-schema";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let Some((operation, options)) = args.split_first() else {
        anyhow::bail!(USAGE);
    };
    if matches!(operation.as_str(), "--help" | "-h") {
        return success_line(USAGE);
    }
    match operation.as_str() {
        "admit" => admit(workspace, options),
        "block" => record_absence(
            workspace,
            options,
            bijux_dna_containers::api::ExternalArtifactAdmissionStatus::BlockedExternalAccess,
        ),
        "unavailable" => record_absence(
            workspace,
            options,
            bijux_dna_containers::api::ExternalArtifactAdmissionStatus::Unavailable,
        ),
        "verify" => verify(workspace, options),
        "bind" => bind(workspace, options),
        "generate-schema" => generate_schema(workspace, options),
        "check-schema" => check_schema(workspace, options),
        _ => anyhow::bail!(USAGE),
    }
}

fn admit(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [repository_root, active_root, repository_commit, tool_id, kind, artifact, sha256, access_basis, recorded_at_utc, output, runtime @ ..] =
        args
    else {
        anyhow::bail!(USAGE);
    };
    let kind = parse_kind(kind)?;
    let apptainer = match (kind, runtime) {
        (bijux_dna_containers::api::ExternalArtifactKind::Binary, []) => None,
        (bijux_dna_containers::api::ExternalArtifactKind::ApptainerSif, [runtime]) => {
            Some(path_from_arg(workspace, runtime))
        }
        _ => anyhow::bail!(USAGE),
    };
    let repository_root = path_from_arg(workspace, repository_root);
    let active_root = path_from_arg(workspace, active_root);
    let artifact = path_from_arg(workspace, artifact);
    let output = path_from_arg(workspace, output);
    let record = bijux_dna_containers::api::admit_external_artifact(
        &bijux_dna_containers::api::ExternalArtifactAdmissionRequest {
            repository_root: &repository_root,
            distributable_image_root: &active_root,
            repository_commit,
            tool_id,
            recorded_at_utc,
            availability: bijux_dna_containers::api::ExternalArtifactAdmissionStatus::Admitted,
            artifact: Some(bijux_dna_containers::api::ExternalArtifactInput {
                kind,
                path: &artifact,
                expected_sha256: sha256,
                lawful_access_basis: access_basis,
                apptainer_executable: apptainer.as_deref(),
            }),
            blocker_reason: None,
        },
        &output,
    )?;
    success_line(format!(
        "admitted external {} artifact for {} at {}",
        kind.as_str(),
        record.tool_id,
        output.display()
    ))
}

fn record_absence(
    workspace: &Workspace,
    args: &[String],
    availability: bijux_dna_containers::api::ExternalArtifactAdmissionStatus,
) -> Result<ContainerCommandOutcome> {
    let [repository_root, active_root, repository_commit, tool_id, recorded_at_utc, output, reason] =
        args
    else {
        anyhow::bail!(USAGE);
    };
    let repository_root = path_from_arg(workspace, repository_root);
    let active_root = path_from_arg(workspace, active_root);
    let output = path_from_arg(workspace, output);
    let record = bijux_dna_containers::api::admit_external_artifact(
        &bijux_dna_containers::api::ExternalArtifactAdmissionRequest {
            repository_root: &repository_root,
            distributable_image_root: &active_root,
            repository_commit,
            tool_id,
            recorded_at_utc,
            availability,
            artifact: None,
            blocker_reason: Some(reason),
        },
        &output,
    )?;
    success_line(format!(
        "recorded external artifact availability {:?} for {} at {}",
        record.availability,
        record.tool_id,
        output.display()
    ))
}

fn verify(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [repository_root, active_root, record] = args else {
        anyhow::bail!(USAGE);
    };
    let record = bijux_dna_containers::api::verify_external_artifact_admission(
        &path_from_arg(workspace, repository_root),
        &path_from_arg(workspace, active_root),
        &path_from_arg(workspace, record),
    )?;
    success_line(format!(
        "verified external artifact record for {} with status {:?}",
        record.tool_id, record.availability
    ))
}

fn bind(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [repository_root, active_root, record, stage_id, apptainer, command @ ..] = args else {
        anyhow::bail!(USAGE);
    };
    if command.is_empty() {
        anyhow::bail!(USAGE);
    }
    let apptainer = (apptainer != "-").then(|| path_from_arg(workspace, apptainer));
    let binding = bijux_dna_containers::api::bind_external_artifact_command(
        &path_from_arg(workspace, repository_root),
        &path_from_arg(workspace, active_root),
        &path_from_arg(workspace, record),
        stage_id,
        command,
        apptainer.as_deref(),
    )?;
    success_line(serde_json::to_string_pretty(&binding)?)
}

fn generate_schema(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let output = match args {
        [] => workspace.path(SCHEMA_PATH),
        [output] => path_from_arg(workspace, output),
        _ => anyhow::bail!(USAGE),
    };
    bijux_dna_containers::api::write_external_artifact_admission_schema(&output)?;
    success_line(format!("generated external-artifact schema at {}", output.display()))
}

fn check_schema(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if !args.is_empty() {
        anyhow::bail!(USAGE);
    }
    let checked: serde_json::Value =
        serde_json::from_str(&read_utf8(&workspace.path(SCHEMA_PATH))?)?;
    if checked != bijux_dna_containers::api::external_artifact_admission_schema() {
        return Ok(ContainerCommandOutcome::failure(
            "external-artifact schema drifted from Rust environment types\n",
        ));
    }
    success_line("external-artifact schema: OK")
}

fn parse_kind(value: &str) -> Result<bijux_dna_containers::api::ExternalArtifactKind> {
    match value {
        "binary" => Ok(bijux_dna_containers::api::ExternalArtifactKind::Binary),
        "apptainer_sif" => Ok(bijux_dna_containers::api::ExternalArtifactKind::ApptainerSif),
        _ => anyhow::bail!("unsupported external artifact kind: {value}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_kinds_are_fail_closed() -> Result<()> {
        assert_eq!(parse_kind("binary")?, bijux_dna_containers::api::ExternalArtifactKind::Binary);
        assert_eq!(
            parse_kind("apptainer_sif")?,
            bijux_dna_containers::api::ExternalArtifactKind::ApptainerSif
        );
        assert!(parse_kind("container").is_err());
        Ok(())
    }

    #[test]
    fn bind_requires_a_stage_command() {
        let workspace = Workspace { root: "/workspace".into() };
        let args = ["/repository", "/active", "/record", "bam.sex", "-"].map(str::to_string);
        assert!(bind(&workspace, &args).is_err());
    }
}
