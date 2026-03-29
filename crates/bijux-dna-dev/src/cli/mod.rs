use std::time::Instant;

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use serde_json::json;

use crate::application::checks::CheckApplication;
use crate::application::containers::ContainerApplication;
use crate::application::domain::DomainApplication;
use crate::application::ops::OpsApplication;
use crate::catalog::ops::{
    assets_registry, docs_registry, examples_registry, hpc_registry, lab_registry, smoke_registry,
    test_registry, tooling_registry,
};
use crate::model::check::{CheckOutcome, CheckSelection, CheckStatus};
use crate::runtime::workspace::Workspace;

mod schema;

use self::schema::*;

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Assets(command) => run_ops("assets", command, assets_registry),
        Command::Checks(command) => run_checks(command),
        Command::Containers(command) => run_containers(command),
        Command::Domain(command) => run_domain(command),
        Command::Docs(command) => run_ops("docs", command, docs_registry),
        Command::Examples(command) => run_ops("examples", command, examples_registry),
        Command::Hpc(command) => run_ops("hpc", command, hpc_registry),
        Command::Lab(command) => run_ops("lab", command, lab_registry),
        Command::Smoke(command) => run_ops("smoke", command, smoke_registry),
        Command::Test(command) => run_ops("test", command, test_registry),
        Command::Tooling(command) => run_ops("tooling", command, tooling_registry),
    }
}

fn maybe_emit_native_help(group: &str, id: &str, err: &anyhow::Error) -> bool {
    let message = err.to_string();
    if !message.starts_with("__help__:") {
        return false;
    }
    println!("Usage: cargo run -p bijux-dna-dev -- {group} run {id} -- [args...]");
    true
}

fn run_checks(command: ChecksCommand) -> Result<()> {
    let app = CheckApplication::new()?;
    match command.command {
        ChecksSubcommand::List => {
            for check in app.registry() {
                println!("{}\tv{}\t{}", check.id, check.version, check.summary);
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

fn run_containers(command: ContainersCommand) -> Result<()> {
    let app = ContainerApplication::new()?;
    match command.command {
        ContainersSubcommand::List => {
            for command in app.registry()? {
                println!("{}\t{}", command.id, command.summary);
            }
            Ok(())
        }
        ContainersSubcommand::Run { id, args } => with_timing("containers", &id, || {
            let outcome = app.run(&id, &args)?;
            if !outcome.stdout.is_empty() {
                print!("{}", outcome.stdout);
            }
            if !outcome.stderr.is_empty() {
                eprint!("{}", outcome.stderr);
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

fn run_domain(command: DomainCommand) -> Result<()> {
    let app = DomainApplication::new()?;
    match command.command {
        DomainSubcommand::List => {
            for command in app.registry() {
                println!("{}\t{}", command.id, command.summary);
            }
            Ok(())
        }
        DomainSubcommand::Run { id, args } => with_timing("domain", &id, || {
            let outcome = app.run(&id, &args)?;
            if !outcome.stdout.is_empty() {
                print!("{}", outcome.stdout);
            }
            if !outcome.stderr.is_empty() {
                eprint!("{}", outcome.stderr);
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

fn run_ops(
    group: &str,
    command: OpsCommand,
    registry: fn() -> Vec<crate::model::ops::OpsCommandDefinition>,
) -> Result<()> {
    let app = OpsApplication::new(registry)?;
    match command.command {
        OpsSubcommand::List => {
            for command in app.registry() {
                println!("{}\t{}", command.id, command.summary);
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
                print!("{}", outcome.stdout);
            }
            if !outcome.stderr.is_empty() {
                eprint!("{}", outcome.stderr);
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

fn print_check_outcome(outcome: &CheckOutcome, depth: usize) {
    let indent = "  ".repeat(depth);
    let status = match outcome.status {
        CheckStatus::Passed => "passed",
        CheckStatus::Failed => "failed",
    };
    println!("{indent}{}: {}", outcome.id, status);
    if outcome.status == CheckStatus::Failed && !outcome.detail.trim().is_empty() {
        println!("{indent}{}", outcome.detail.trim());
    }
    for child in &outcome.children {
        print_check_outcome(child, depth + 1);
    }
}

fn with_timing<F>(group: &str, command: &str, action: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    let started_at = Utc::now();
    let timer = Instant::now();
    let result = action();
    let ended_at = Utc::now();
    let exit_code = i32::from(result.is_err());
    let status = if exit_code == 0 { "ok" } else { "fail" };
    write_timing(
        group,
        command,
        status,
        exit_code,
        started_at,
        ended_at,
        timer.elapsed().as_secs(),
    );
    result
}

fn write_timing(
    group: &str,
    command: &str,
    status: &str,
    exit_code: i32,
    started_at: chrono::DateTime<Utc>,
    ended_at: chrono::DateTime<Utc>,
    duration_seconds: u64,
) {
    let Ok(workspace) = Workspace::resolve() else {
        return;
    };
    let timing_dir = std::env::var("ARTIFACT_DIR")
        .ok()
        .or_else(|| std::env::var("ISO_ROOT").ok())
        .map(|raw| {
            let path = std::path::PathBuf::from(raw);
            if path.is_absolute() {
                path.join("timing")
            } else {
                workspace.root.join(path).join("timing")
            }
        })
        .unwrap_or_else(|| workspace.path("artifacts/timing"));
    if bijux_dna_infra::ensure_dir(&timing_dir).is_err() {
        return;
    }
    let file_name = format!("{}__{}.json", group, command.replace('/', "_"));
    let payload = json!({
        "group": group,
        "command": command,
        "status": status,
        "exit_code": exit_code,
        "start_utc": started_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "end_utc": ended_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "duration_seconds": duration_seconds,
    });
    let _ = bijux_dna_infra::write_bytes(
        timing_dir.join(file_name),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        ),
    );
}
