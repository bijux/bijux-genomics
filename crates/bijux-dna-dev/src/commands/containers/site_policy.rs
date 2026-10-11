use std::path::PathBuf;

use anyhow::Context as _;

use super::{
    command_hostname, path_from_arg, success_line, ContainerCommandOutcome, Result, Workspace,
};

const FREEZE_USAGE: &str = "Usage: bijux-dna-dev containers run repository-freeze -- --governance <path> --commit <sha> [--scope <contracts|sources|complete>] [--path <repository-relative-path>]... [--namespace <name>]";
const ARCHIVE_USAGE: &str = "Usage:\n  bijux-dna-dev containers run repository-archive -- archive --governance <path> --snapshot <path> --commit <sha>\n  bijux-dna-dev containers run repository-archive -- verify --governance <path> --snapshot <path> --commit <sha> --receipt <path>\n  bijux-dna-dev containers run repository-archive -- retire --governance <path> --snapshot <path> --commit <sha> --receipt <path>\n  bijux-dna-dev containers run repository-archive -- restore --governance <path> --snapshot <path> --commit <sha> --receipt <path>";

#[derive(Clone, Copy)]
enum ArchiveOperation {
    Archive,
    Verify,
    Retire,
    Restore,
}

pub(super) fn freeze_repository(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    if args.iter().any(|argument| matches!(argument.as_str(), "--help" | "-h")) {
        return success_line(FREEZE_USAGE);
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
        let value = args.get(index + 1).ok_or_else(|| anyhow::anyhow!(FREEZE_USAGE))?;
        match option {
            "--governance" => set_once(&mut governance, value, option)?,
            "--commit" => set_once(&mut commit, value, option)?,
            "--scope" if !scope_set => {
                scope = match value.as_str() {
                    "contracts" => bijux_dna_containers::api::RepositoryFreezeScope::Contracts,
                    "sources" => bijux_dna_containers::api::RepositoryFreezeScope::Sources,
                    "complete" => bijux_dna_containers::api::RepositoryFreezeScope::Complete,
                    _ => anyhow::bail!(FREEZE_USAGE),
                };
                scope_set = true;
            }
            "--path" => checkout_paths.push(PathBuf::from(value)),
            "--namespace" if !namespace_set => {
                namespace.clone_from(value);
                namespace_set = true;
            }
            _ => anyhow::bail!(FREEZE_USAGE),
        }
        index += 2;
    }
    let governance_path = path_from_arg(
        workspace,
        governance.as_deref().ok_or_else(|| anyhow::anyhow!(FREEZE_USAGE))?,
    );
    let governance = bijux_dna_containers::api::load_container_governance(&governance_path)?;
    let target = bijux_dna_containers::api::freeze_repository(
        &bijux_dna_containers::api::RepositoryFreezeRequest {
            governance,
            hostname: command_hostname(),
            commit: commit.ok_or_else(|| anyhow::anyhow!(FREEZE_USAGE))?,
            scope,
            checkout_paths,
            source_namespace: namespace,
        },
    )?;
    success_line(target.display().to_string())
}

pub(super) fn repository_archive(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    if args.iter().any(|argument| matches!(argument.as_str(), "--help" | "-h")) {
        return success_line(ARCHIVE_USAGE);
    }
    let (operation, options) = args.split_first().ok_or_else(|| anyhow::anyhow!(ARCHIVE_USAGE))?;
    let operation = match operation.as_str() {
        "archive" => ArchiveOperation::Archive,
        "verify" => ArchiveOperation::Verify,
        "retire" => ArchiveOperation::Retire,
        "restore" => ArchiveOperation::Restore,
        _ => anyhow::bail!(ARCHIVE_USAGE),
    };
    let mut governance = None;
    let mut snapshot = None;
    let mut commit = None;
    let mut receipt = None;
    let mut index = 0;
    while index < options.len() {
        let option = options[index].as_str();
        let value = options.get(index + 1).ok_or_else(|| anyhow::anyhow!(ARCHIVE_USAGE))?;
        match option {
            "--governance" => set_once(&mut governance, value, option)?,
            "--snapshot" => set_once(&mut snapshot, value, option)?,
            "--commit" => set_once(&mut commit, value, option)?,
            "--receipt" => set_once(&mut receipt, value, option)?,
            _ => anyhow::bail!(ARCHIVE_USAGE),
        }
        index += 2;
    }
    if matches!(operation, ArchiveOperation::Archive) {
        ensure_no_receipt(receipt.as_deref())?;
    }
    let governance_path = path_from_arg(
        workspace,
        governance.as_deref().ok_or_else(|| anyhow::anyhow!(ARCHIVE_USAGE))?,
    );
    let request = bijux_dna_containers::api::RepositoryFreezeArchiveRequest {
        governance: bijux_dna_containers::api::load_container_governance(&governance_path)?,
        hostname: command_hostname(),
        snapshot: path_from_arg(
            workspace,
            snapshot.as_deref().ok_or_else(|| anyhow::anyhow!(ARCHIVE_USAGE))?,
        ),
        commit: commit.ok_or_else(|| anyhow::anyhow!(ARCHIVE_USAGE))?,
    };
    match operation {
        ArchiveOperation::Archive => {
            let archived = bijux_dna_containers::api::archive_frozen_repository(&request)?;
            success_line(serde_json::to_string_pretty(&archived)?)
        }
        ArchiveOperation::Verify | ArchiveOperation::Retire | ArchiveOperation::Restore => {
            let receipt_path = path_from_arg(
                workspace,
                receipt.as_deref().ok_or_else(|| anyhow::anyhow!(ARCHIVE_USAGE))?,
            );
            let archived: bijux_dna_containers::api::RepositoryFreezeArchiveReceipt =
                serde_json::from_str(&std::fs::read_to_string(&receipt_path).with_context(
                    || format!("read repository archive receipt {}", receipt_path.display()),
                )?)
                .with_context(|| {
                    format!("parse repository archive receipt {}", receipt_path.display())
                })?;
            match operation {
                ArchiveOperation::Verify => {
                    bijux_dna_containers::api::verify_frozen_repository_archive(
                        &request, &archived,
                    )?;
                    success_line("verified")
                }
                ArchiveOperation::Retire => {
                    bijux_dna_containers::api::retire_frozen_repository(&request, &archived)?;
                    success_line("retired")
                }
                ArchiveOperation::Restore => {
                    let restored = bijux_dna_containers::api::restore_frozen_repository_archive(
                        &request, &archived,
                    )?;
                    success_line(restored.display().to_string())
                }
                ArchiveOperation::Archive => unreachable!("archive handled before receipt loading"),
            }
        }
    }
}

fn ensure_no_receipt(receipt: Option<&str>) -> Result<()> {
    if receipt.is_some() {
        anyhow::bail!("--receipt is invalid for archive; {ARCHIVE_USAGE}");
    }
    Ok(())
}

fn set_once(target: &mut Option<String>, value: &str, option: &str) -> Result<()> {
    if target.replace(value.to_string()).is_some() {
        anyhow::bail!("{option} may be supplied only once");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_help_exposes_every_reversible_operation() -> Result<()> {
        let workspace = Workspace { root: PathBuf::from("/workspace") };
        let output = repository_archive(&workspace, &["--help".into()])?;
        for operation in ["archive", "verify", "retire", "restore"] {
            assert!(output.stdout.contains(operation));
        }
        Ok(())
    }

    #[test]
    fn archive_rejects_a_receipt_before_touching_site_state() {
        let workspace = Workspace { root: PathBuf::from("/workspace") };
        let result = repository_archive(
            &workspace,
            &[
                "archive".into(),
                "--governance".into(),
                "missing-governance.json".into(),
                "--snapshot".into(),
                "missing-snapshot".into(),
                "--commit".into(),
                "0".repeat(40),
                "--receipt".into(),
                "forbidden.json".into(),
            ],
        );
        assert!(result.is_err_and(|error| error.to_string().contains("invalid for archive")));
    }
}
