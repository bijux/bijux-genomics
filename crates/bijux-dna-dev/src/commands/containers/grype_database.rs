use super::{path_from_arg, success_line, ContainerCommandOutcome, Result, Workspace};

const USAGE: &str =
    "Usage:\n  bijux-dna-dev containers run grype-database -- <contract> <scanner-root> <evidence-root>";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if matches!(args, [help] if matches!(help.as_str(), "--help" | "-h")) {
        return success_line(USAGE);
    }
    let [contract, scanner_root, evidence_root] = args else {
        anyhow::bail!(USAGE);
    };
    let admitted = bijux_dna_containers::api::verify_grype_database(
        &path_from_arg(workspace, contract),
        &path_from_arg(workspace, scanner_root),
        &path_from_arg(workspace, evidence_root),
    )?;
    success_line(serde_json::to_string_pretty(&admitted)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_requires_exact_admission_inputs() {
        let workspace = Workspace { root: "/workspace".into() };
        assert!(run(&workspace, &[]).is_err());
        assert!(run(&workspace, &["contract.toml".into()]).is_err());
        assert!(run(
            &workspace,
            &["contract.toml".into(), "scanner".into(), "evidence".into(), "extra".into(),],
        )
        .is_err());
        assert!(run(&workspace, &["--help".into()]).is_ok());
    }
}
