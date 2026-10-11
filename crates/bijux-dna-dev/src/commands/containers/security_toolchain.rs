use super::{path_from_arg, success_line, ContainerCommandOutcome, Result, Workspace};

const USAGE: &str =
    "Usage:\n  bijux-dna-dev containers run security-toolchain -- <contract> <evidence-root>";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if matches!(args, [help] if matches!(help.as_str(), "--help" | "-h")) {
        return success_line(USAGE);
    }
    let [contract, evidence_root] = args else {
        anyhow::bail!(USAGE);
    };
    let completion = bijux_dna_containers::api::verify_security_scanner_toolchain(
        &path_from_arg(workspace, contract),
        &path_from_arg(workspace, evidence_root),
    )?;
    success_line(serde_json::to_string_pretty(&completion)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_requires_exact_admission_inputs() {
        let workspace = Workspace { root: "/workspace".into() };
        assert!(run(&workspace, &[]).is_err());
        assert!(run(&workspace, &["contract.toml".into()]).is_err());
        assert!(
            run(&workspace, &["contract.toml".into(), "evidence".into(), "extra".into()]).is_err()
        );
        assert!(run(&workspace, &["--help".into()]).is_ok());
    }
}
