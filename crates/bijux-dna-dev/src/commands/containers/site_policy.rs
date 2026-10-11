use std::path::PathBuf;

use super::{
    command_hostname, path_from_arg, success_line, ContainerCommandOutcome, Result, Workspace,
};

const USAGE: &str = "Usage: bijux-dna-dev containers run repository-freeze -- --governance <path> --commit <sha> [--scope <contracts|sources|complete>] [--path <repository-relative-path>]... [--namespace <name>]";

pub(super) fn freeze_repository(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    if args.iter().any(|argument| matches!(argument.as_str(), "--help" | "-h")) {
        return success_line(USAGE);
    }
    let mut governance = None;
    let mut commit = None;
    let mut scope = bijux_dna_containers::api::RepositoryFreezeScope::Contracts;
    let mut scope_set = false;
    let mut checkout_paths = Vec::new();
    let mut namespace = "source-freezes".to_string();
    let mut namespace_set = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index].as_str();
        let value = args.get(index + 1).ok_or_else(|| anyhow::anyhow!(USAGE))?;
        match option {
            "--governance" => set_once(&mut governance, value, option)?,
            "--commit" => set_once(&mut commit, value, option)?,
            "--scope" if !scope_set => {
                scope = match value.as_str() {
                    "contracts" => bijux_dna_containers::api::RepositoryFreezeScope::Contracts,
                    "sources" => bijux_dna_containers::api::RepositoryFreezeScope::Sources,
                    "complete" => bijux_dna_containers::api::RepositoryFreezeScope::Complete,
                    _ => anyhow::bail!(USAGE),
                };
                scope_set = true;
            }
            "--path" => checkout_paths.push(PathBuf::from(value)),
            "--namespace" if !namespace_set => {
                namespace.clone_from(value);
                namespace_set = true;
            }
            _ => anyhow::bail!(USAGE),
        }
        index += 2;
    }
    let governance_path =
        path_from_arg(workspace, governance.as_deref().ok_or_else(|| anyhow::anyhow!(USAGE))?);
    let governance = bijux_dna_containers::api::load_container_governance(&governance_path)?;
    let target = bijux_dna_containers::api::freeze_repository(
        &bijux_dna_containers::api::RepositoryFreezeRequest {
            governance,
            hostname: command_hostname(),
            commit: commit.ok_or_else(|| anyhow::anyhow!(USAGE))?,
            scope,
            checkout_paths,
            source_namespace: namespace,
        },
    )?;
    success_line(target.display().to_string())
}

fn set_once(target: &mut Option<String>, value: &str, option: &str) -> Result<()> {
    if target.replace(value.to_string()).is_some() {
        anyhow::bail!("{option} may be supplied only once");
    }
    Ok(())
}
