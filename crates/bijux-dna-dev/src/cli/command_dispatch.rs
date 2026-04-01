use anyhow::Result;

use crate::application::checks::CheckApplication;
use crate::application::containers::ContainerApplication;
use crate::application::domain::DomainApplication;
use crate::application::ops::OpsApplication;
use crate::model::check::{CheckOutcome, CheckSelection, CheckStatus};

use super::execution_reporting::{
    maybe_emit_native_help, print_check_outcome, with_timing, write_line_stdout, write_stderr,
    write_stdout,
};
use super::schema::{
    ChecksCommand, ChecksSubcommand, ContainersCommand, ContainersSubcommand, DomainCommand,
    DomainSubcommand, OpsCommand, OpsSubcommand,
};

pub(super) fn run_checks(command: ChecksCommand) -> Result<()> {
    let app = CheckApplication::new()?;
    match command.command {
        ChecksSubcommand::List => {
            for check in CheckApplication::registry() {
                write_line_stdout(&format!(
                    "{}\tv{}\t{}",
                    check.id, check.version, check.summary
                ))?;
            }
            Ok(())
        }
        ChecksSubcommand::Run { all, id } => {
            let command_id = if all {
                "all".to_string()
            } else {
                id.clone().unwrap_or_default()
            };
            with_timing("checks", &command_id, || {
                let selection = if all {
                    CheckSelection::All
                } else {
                    CheckSelection::Single(
                        id.ok_or_else(|| anyhow::anyhow!("checks run requires <id> or --all"))?,
                    )
                };
                let outcomes = app.run_selection(selection)?;
                let mut failed = false;
                for outcome in outcomes {
                    if outcome.status == CheckStatus::Failed {
                        failed = true;
                    }
                    print_check_outcome(&outcome, 0);
                }
                if failed {
                    anyhow::bail!("one or more checks failed");
                }
                Ok(())
            })
        }
    }
}

pub(super) fn run_containers(command: ContainersCommand) -> Result<()> {
    let app = ContainerApplication::new()?;
    match command.command {
        ContainersSubcommand::List => {
            for command in app.registry()? {
                write_line_stdout(&format!("{}\t{}", command.id, command.summary))?;
            }
            Ok(())
        }
        ContainersSubcommand::Run { id, args } => with_timing("containers", &id, || {
            let outcome = app.run(&id, &args)?;
            if !outcome.stdout.is_empty() {
                write_stdout(&outcome.stdout)?;
            }
            if !outcome.stderr.is_empty() {
                write_stderr(&outcome.stderr)?;
            }
            if !outcome.is_success() {
                anyhow::bail!(
                    "container command `{id}` failed with exit code {}",
                    outcome.exit_code
                );
            }
            Ok(())
        }),
    }
}

pub(super) fn run_domain(command: DomainCommand) -> Result<()> {
    let app = DomainApplication::new()?;
    match command.command {
        DomainSubcommand::List => {
            for command in DomainApplication::registry() {
                write_line_stdout(&format!("{}\t{}", command.id, command.summary))?;
            }
            Ok(())
        }
        DomainSubcommand::Run { id, args } => with_timing("domain", &id, || {
            let outcome = app.run(&id, &args)?;
            if !outcome.stdout.is_empty() {
                write_stdout(&outcome.stdout)?;
            }
            if !outcome.stderr.is_empty() {
                write_stderr(&outcome.stderr)?;
            }
            if !outcome.is_success() {
                anyhow::bail!(
                    "domain command `{id}` failed with exit code {}",
                    outcome.exit_code
                );
            }
            Ok(())
        }),
    }
}

pub(super) fn run_ops(
    group: &str,
    command: OpsCommand,
    registry: fn() -> Vec<crate::model::ops::OpsCommandDefinition>,
) -> Result<()> {
    let app = OpsApplication::new(registry)?;
    match command.command {
        OpsSubcommand::List => {
            for command in app.registry() {
                write_line_stdout(&format!("{}\t{}", command.id, command.summary))?;
            }
            Ok(())
        }
        OpsSubcommand::Run { id, args } => with_timing(group, &id, || {
            let outcome = match app.run(&id, &args) {
                Ok(outcome) => outcome,
                Err(err) if maybe_emit_native_help(group, &id, &err) => return Ok(()),
                Err(err) => return Err(err),
            };
            if !outcome.stdout.is_empty() {
                write_stdout(&outcome.stdout)?;
            }
            if !outcome.stderr.is_empty() {
                write_stderr(&outcome.stderr)?;
            }
            if !outcome.is_success() {
                anyhow::bail!(
                    "{group} command `{id}` failed with exit code {}",
                    outcome.exit_code
                );
            }
            Ok(())
        }),
    }
}
