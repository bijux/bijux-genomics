use super::{path_from_arg, success_line, ContainerCommandOutcome, Result, Workspace};

const USAGE: &str = "Usage:\n  bijux-dna-dev containers run image-campaign -- verify <repository-root> <study-root> <apptainer> <attempt-root>";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if matches!(args, [help] if matches!(help.as_str(), "--help" | "-h")) {
        return success_line(USAGE);
    }
    let [operation, repository_root, study_root, apptainer, attempt_root] = args else {
        anyhow::bail!(USAGE);
    };
    if operation != "verify" {
        anyhow::bail!(USAGE);
    }
    let execution = bijux_dna_containers::api::verify_apptainer_image_campaign_receipt(
        &path_from_arg(workspace, repository_root),
        &path_from_arg(workspace, study_root),
        &path_from_arg(workspace, apptainer),
        &path_from_arg(workspace, attempt_root),
    )?;
    success_line(serde_json::to_string_pretty(&execution)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_requires_exact_verification_inputs() {
        let workspace = Workspace { root: "/workspace".into() };
        assert!(run(&workspace, &[]).is_err());
        assert!(run(&workspace, &["verify".into()]).is_err());
        assert!(run(
            &workspace,
            &[
                "admit".into(),
                "repository".into(),
                "study".into(),
                "apptainer".into(),
                "attempt".into(),
            ],
        )
        .is_err());
        assert!(run(&workspace, &["--help".into()]).is_ok());
    }
}
